use lsys_core::fluent_message;
use redis::AsyncCommands;

use super::types::UploadTokenData;
use super::UploadTokenManager;
use crate::dao::result::{FileManagerError, FileManagerResult};

impl UploadTokenManager {
    /// 签发单文件短时令牌。
    ///
    /// 若该 `file_ref_id` 已存在活跃凭证（令牌或分片会话），先清理旧凭证。
    pub async fn create_token(
        &self,
        file_ref_id: u64,
        user_id: u64,
        app_id: u64,
        expire_secs: Option<u64>,
    ) -> FileManagerResult<String> {
        let token = Self::gen_id();
        let ttl = expire_secs.unwrap_or(super::TOKEN_DEFAULT_EXPIRE_SECS);

        let data = UploadTokenData {
            file_ref_id,
            user_id,
            app_id,
            session_id: None,
            part_number: None,
        };
        let value = serde_json::to_string(&data)
            .map_err(|e| FileManagerError::Message(fluent_message!("upload-token-serialize-error", e)))?;

        let mut conn = self.redis.get().await?;

        Self::replace_ref(&mut conn, file_ref_id, &Self::ref_value_token(&token), ttl).await?;
        conn.set_ex::<_, _, ()>(&Self::token_key(&token), &value, ttl).await?;

        Ok(token)
    }

    /// 解析单文件短时令牌，返回绑定数据。
    pub async fn resolve_token(&self, token: &str) -> FileManagerResult<UploadTokenData> {
        let key = Self::token_key(token);
        let mut conn = self.redis.get().await?;

        let raw: Option<String> = conn.get(&key).await?;
        let raw = raw.ok_or_else(|| {
            FileManagerError::Message(fluent_message!("upload-token-invalid-or-expired"))
        })?;

        serde_json::from_str::<UploadTokenData>(&raw)
            .map_err(|e| FileManagerError::Message(fluent_message!("upload-token-parse-error", e)))
    }

    /// 销毁单文件令牌（上传完成或失败终态时调用）。
    pub async fn remove_token(&self, token: &str) -> FileManagerResult<()> {
        let token_key = Self::token_key(token);
        let mut conn = self.redis.get().await?;

        if let Some(raw) = conn.get::<_, Option<String>>(&token_key).await? {
            let data = serde_json::from_str::<UploadTokenData>(&raw)
                .map_err(|e| FileManagerError::Message(fluent_message!("upload-token-parse-error", e)))?;
            Self::try_remove_ref(
                &mut conn,
                data.file_ref_id,
                &Self::ref_value_token(token),
            )
            .await;
        }

        conn.del::<_, ()>(&token_key).await?;
        Ok(())
    }
}
