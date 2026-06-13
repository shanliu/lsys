use lsys_core::fluent_message;
use redis::AsyncCommands;
use tracing::warn;

use super::types::{UploadSessionData, UploadTokenData};
use super::UploadTokenManager;
use crate::dao::result::{FileManagerError, FileManagerResult};

impl UploadTokenManager {
    /// 创建分片上传长时会话（对应 S3 CreateMultipartUpload）。
    ///
    /// 若该 `file_ref_id` 已存在活跃凭证（含旧会话），先清理。
    pub async fn create_session(
        &self,
        file_ref_id: u64,
        user_id: u64,
        app_id: u64,
        total_parts: u32,
        expire_secs: Option<u64>,
    ) -> FileManagerResult<String> {
        let session_id = Self::gen_id();
        let ttl = expire_secs.unwrap_or(super::SESSION_DEFAULT_EXPIRE_SECS);

        let data = UploadSessionData {
            file_ref_id,
            user_id,
            app_id,
            total_parts,
        };
        let value = serde_json::to_string(&data)
            .map_err(|e| FileManagerError::Message(fluent_message!("upload-token-serialize-error", e)))?;

        let mut conn = self.redis.get().await?;

        Self::replace_ref(
            &mut conn,
            file_ref_id,
            &Self::ref_value_session(&session_id),
            ttl,
        )
        .await?;
        conn.set_ex::<_, _, ()>(&Self::session_key(&session_id), &value, ttl).await?;

        Ok(session_id)
    }

    /// 解析分片上传会话，返回会话数据。
    pub async fn resolve_session(&self, session_id: &str) -> FileManagerResult<UploadSessionData> {
        let key = Self::session_key(session_id);
        let mut conn = self.redis.get().await?;

        let raw: Option<String> = conn.get(&key).await?;
        let raw = raw.ok_or_else(|| {
            FileManagerError::Message(fluent_message!("upload-token-invalid-or-expired"))
        })?;

        Self::parse_session(&raw)
    }

    /// 为指定会话的某个分片签发短时令牌（对应 S3 UploadPart Presigned URL）。
    ///
    /// 会话必须存在，分片令牌 TTL 取请求值与会话剩余 TTL 中的较小值。
    pub async fn create_part_token(
        &self,
        session_id: &str,
        part_number: u32,
        expire_secs: Option<u64>,
    ) -> FileManagerResult<String> {
        let session_key = Self::session_key(session_id);
        let mut conn = self.redis.get().await?;

        // 校验会话存在
        let raw: Option<String> = conn.get(&session_key).await?;
        let raw = raw.ok_or_else(|| {
            FileManagerError::Message(fluent_message!("upload-token-invalid-or-expired"))
        })?;
        let session = Self::parse_session(&raw)?;

        // 分片令牌 TTL：不超过会话剩余时间
        let session_remaining: i64 = conn.ttl(&session_key).await?;
        let requested_ttl = expire_secs.unwrap_or(super::TOKEN_DEFAULT_EXPIRE_SECS);
        let ttl = if session_remaining > 0 {
            requested_ttl.min(session_remaining as u64)
        } else {
            requested_ttl
        };

        let token = Self::gen_id();
        let data = UploadTokenData {
            file_ref_id: session.file_ref_id,
            user_id: session.user_id,
            app_id: session.app_id,
            session_id: Some(session_id.to_owned()),
            part_number: Some(part_number),
        };
        let value = serde_json::to_string(&data)
            .map_err(|e| FileManagerError::Message(fluent_message!("upload-token-serialize-error", e)))?;

        conn.set_ex::<_, _, ()>(&Self::token_key(&token), &value, ttl).await?;

        // 将分片令牌加入会话 parts set，用于 remove_session 批量清理
        let parts_key = Self::session_parts_key(session_id);
        conn.sadd::<_, _, ()>(&parts_key, &token).await?;
        if session_remaining > 0 {
            if let Err(e) = conn.expire::<_, ()>(&parts_key, session_remaining).await {
                warn!("upload-token: failed to set TTL on parts set '{}': {e}", parts_key);
            }
        }

        Ok(token)
    }

    /// 中止分片上传：删除会话及其全部分片令牌，清理反向索引。
    pub async fn remove_session(&self, session_id: &str) -> FileManagerResult<()> {
        let session_key = Self::session_key(session_id);
        let mut conn = self.redis.get().await?;

        if let Some(raw) = conn.get::<_, Option<String>>(&session_key).await? {
            let data = Self::parse_session(&raw)?;
            Self::try_remove_ref(
                &mut conn,
                data.file_ref_id,
                &Self::ref_value_session(session_id),
            )
            .await;
        }

        Self::delete_session_keys(&mut conn, session_id).await;
        Ok(())
    }
}
