//! 上传令牌管理模块
//!
//! 负责上传令牌的生成、验证（含自动续期）、销毁和重新签发。
//! 令牌存储在 Redis 中，利用 TTL 天然实现过期清理。
//!
//! 关键设计要点（参见文档 5.6 节"设计要点与常见陷阱"）：
//! - 令牌支持多分片复用：verify 同时接受 status=0 和 status=1
//! - 每次分片验证自动续期 TTL，确保超大文件上传不超时
//! - 令牌仅在整个文件最终完成/失败时销毁，非按分片销毁
//! - 支持 retoken 为未完成文件重新签发令牌

use deadpool_redis::Pool as RedisPool;
use lsys_core::fluent_message;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use crate::dao::result::{WebError, WebResult};

/// Redis key 前缀
pub const UPLOAD_TOKEN_PREFIX: &str = "file:upload_token:";

/// 默认有效期（秒）
pub const UPLOAD_TOKEN_DEFAULT_EXPIRE: u64 = 1800;

/// 令牌 Redis value 结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadTokenData {
    pub file_user_id: u64,
    pub user_id: u64,
    pub app_id: u64,
    /// 0=未使用（刚签发），1=使用中（至少一个分片已开始上传）
    pub status: u8,
}

/// 令牌验证结果
#[derive(Debug, Clone)]
pub struct UploadTokenVerifyResult {
    pub user_id: u64,
    pub app_id: u64,
}

/// 上传令牌 DAO
pub struct UploadTokenDao {
    redis: RedisPool,
}

impl UploadTokenDao {
    pub fn new(redis: RedisPool) -> Self {
        Self { redis }
    }

    /// 生成 Redis key
    fn make_key(token: &str) -> String {
        format!("{}{}", UPLOAD_TOKEN_PREFIX, token)
    }

    /// 生成随机令牌字符串（32 字节 hex = 64 字符）
    fn generate_token() -> String {
        lsys_core::utils::rand_str(lsys_core::utils::RandType::LowerHex, 64)
    }

    /// 生成上传令牌
    ///
    /// 创建一个新的上传令牌并写入 Redis，绑定 file_user_id、user_id、app_id。
    pub async fn create_upload_token(
        &self,
        file_user_id: u64,
        user_id: u64,
        app_id: u64,
        expire_secs: Option<u64>,
    ) -> WebResult<String> {
        let token = Self::generate_token();
        let key = Self::make_key(&token);
        let expire = expire_secs.unwrap_or(UPLOAD_TOKEN_DEFAULT_EXPIRE);

        let data = UploadTokenData {
            file_user_id,
            user_id,
            app_id,
            status: 0,
        };
        let value = serde_json::to_string(&data)
            .map_err(|e| WebError::Message(fluent_message!("upload-token-serialize-error", e)))?;

        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| WebError::Message(fluent_message!("redis-connect-error", e)))?;

        conn.set_ex::<_, _, ()>(&key, &value, expire)
            .await
            .map_err(|e| WebError::Message(fluent_message!("redis-set-error", e)))?;

        Ok(token)
    }

    /// 验证上传令牌（含自动续期）
    ///
    /// 使用 Lua 脚本原子执行：
    /// 1. 读取 key → 校验存在性
    /// 2. 解析 JSON → 校验 file_user_id 匹配
    /// 3. 若 status=0 则更新为 1；若 status=1 则保持
    /// 4. 重置 TTL（自动续期）
    /// 5. 返回 user_id、app_id
    pub async fn verify_upload_token(
        &self,
        token: &str,
        file_user_id: u64,
    ) -> WebResult<UploadTokenVerifyResult> {
        let key = Self::make_key(token);
        let expire = UPLOAD_TOKEN_DEFAULT_EXPIRE;

        // Lua 脚本：原子读取-校验-更新status-续期
        let lua_script = r#"
            local val = redis.call('GET', KEYS[1])
            if not val then
                return redis.error_reply('TOKEN_NOT_FOUND')
            end
            local data = cjson.decode(val)
            if tostring(data.file_user_id) ~= ARGV[1] then
                return redis.error_reply('FILE_USER_ID_MISMATCH')
            end
            if data.status == 0 then
                data.status = 1
                redis.call('SET', KEYS[1], cjson.encode(data))
            end
            redis.call('EXPIRE', KEYS[1], tonumber(ARGV[2]))
            return cjson.encode({user_id = data.user_id, app_id = data.app_id})
        "#;

        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| WebError::Message(fluent_message!("redis-connect-error", e)))?;

        let result: Result<String, redis::RedisError> = redis::Script::new(lua_script)
            .key(&key)
            .arg(file_user_id.to_string())
            .arg(expire)
            .invoke_async(&mut *conn)
            .await;

        match result {
            Ok(json_str) => {
                #[derive(Deserialize)]
                struct LuaResult {
                    #[serde(deserialize_with = "deserialize_number_from_lua")]
                    user_id: u64,
                    #[serde(deserialize_with = "deserialize_number_from_lua")]
                    app_id: u64,
                }
                let parsed: LuaResult = serde_json::from_str(&json_str).map_err(|e| {
                    WebError::Message(fluent_message!("upload-token-parse-error", e))
                })?;
                Ok(UploadTokenVerifyResult {
                    user_id: parsed.user_id,
                    app_id: parsed.app_id,
                })
            }
            Err(e) => {
                let msg = e.to_string();
                if msg.contains("TOKEN_NOT_FOUND") {
                    Err(WebError::Message(fluent_message!(
                        "upload-token-invalid-or-expired"
                    )))
                } else if msg.contains("FILE_USER_ID_MISMATCH") {
                    Err(WebError::Message(fluent_message!(
                        "upload-token-file-id-mismatch"
                    )))
                } else {
                    Err(WebError::Message(fluent_message!("redis-script-error", e)))
                }
            }
        }
    }

    /// 销毁令牌
    ///
    /// 在整个文件最终完成（Normal）或整个文件失败（Failed）时调用。
    /// 多分片场景下单个分片的 complete/fail 不应调用此方法。
    pub async fn consume_upload_token(&self, token: &str) -> WebResult<()> {
        let key = Self::make_key(token);

        let mut conn = self
            .redis
            .get()
            .await
            .map_err(|e| WebError::Message(fluent_message!("redis-connect-error", e)))?;

        conn.del::<_, ()>(&key)
            .await
            .map_err(|e| WebError::Message(fluent_message!("redis-del-error", e)))?;

        Ok(())
    }

    /// 重新签发令牌（断点续传）
    ///
    /// 校验 file_user_id 对应文件状态为 Unfinished、user_id/app_id 匹配后，
    /// 删除可能存在的旧令牌，生成新令牌写入 Redis。
    ///
    /// 注：文件状态校验由调用方（service handler）完成，此处仅处理令牌逻辑。
    pub async fn retoken_upload(
        &self,
        file_user_id: u64,
        user_id: u64,
        app_id: u64,
        old_token: Option<&str>,
        expire_secs: Option<u64>,
    ) -> WebResult<String> {
        // 删除旧令牌（如果存在）
        if let Some(old) = old_token {
            let _ = self.consume_upload_token(old).await;
        }

        // 生成新令牌
        self.create_upload_token(file_user_id, user_id, app_id, expire_secs)
            .await
    }
}

/// 辅助：从 Lua 返回的 JSON 中反序列化数字（cjson 可能返回浮点数）
fn deserialize_number_from_lua<'de, D>(deserializer: D) -> Result<u64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, Visitor};
    use std::fmt;

    struct NumVisitor;

    impl<'de> Visitor<'de> for NumVisitor {
        type Value = u64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a number (integer or float)")
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
            Ok(v as u64)
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<Self::Value, E> {
            Ok(v as u64)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse::<u64>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(NumVisitor)
}
