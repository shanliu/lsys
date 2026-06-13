use tracing::warn;
use redis::AsyncCommands;

use super::types::UploadSessionData;
use super::UploadTokenManager;

impl UploadTokenManager {
    /// 根据 ref 值解析出当前活跃凭证并清理，然后写入新 ref 值。
    ///
    /// 旧凭证清理失败只记 warn（TTL 到期自动消失），新凭证写入失败返回错误。
    pub(super) async fn replace_ref(
        conn: &mut deadpool_redis::Connection,
        file_ref_id: u64,
        new_ref_value: &str,
        ttl: u64,
    ) -> crate::dao::result::FileManagerResult<()> {
        let ref_key = Self::ref_key(file_ref_id);

        if let Some(old_ref) = conn.get::<_, Option<String>>(&ref_key).await? {
            if let Some(old_token) = old_ref.strip_prefix("token:") {
                let key = Self::token_key(old_token);
                if let Err(e) = conn.del::<_, ()>(&key).await {
                    warn!("upload-token: failed to delete old token '{}': {e}", key);
                }
            } else if let Some(old_session_id) = old_ref.strip_prefix("session:") {
                Self::delete_session_keys(conn, old_session_id).await;
            }
        }

        conn.set_ex::<_, _, ()>(&ref_key, new_ref_value, ttl).await?;
        Ok(())
    }

    /// 删除会话主键及其全部分片令牌键，失败只 warn。
    pub(super) async fn delete_session_keys(
        conn: &mut deadpool_redis::Connection,
        session_id: &str,
    ) {
        let parts_key = Self::session_parts_key(session_id);

        match conn.smembers::<_, Vec<String>>(&parts_key).await {
            Ok(tokens) => {
                for t in &tokens {
                    let key = Self::token_key(t);
                    if let Err(e) = conn.del::<_, ()>(&key).await {
                        warn!("upload-token: failed to delete part token '{}': {e}", key);
                    }
                }
                if let Err(e) = conn.del::<_, ()>(&parts_key).await {
                    warn!("upload-token: failed to delete parts set '{}': {e}", parts_key);
                }
            }
            Err(e) => {
                warn!("upload-token: failed to load parts set '{}': {e}", parts_key);
            }
        }

        let session_key = Self::session_key(session_id);
        if let Err(e) = conn.del::<_, ()>(&session_key).await {
            warn!("upload-token: failed to delete session '{}': {e}", session_key);
        }
    }

    /// 清理反向索引（仅当索引仍指向期望值时删除），失败只 warn。
    pub(super) async fn try_remove_ref(
        conn: &mut deadpool_redis::Connection,
        file_ref_id: u64,
        expected_ref_value: &str,
    ) {
        let ref_key = Self::ref_key(file_ref_id);
        match conn.get::<_, Option<String>>(&ref_key).await {
            Ok(Some(cur)) if cur == expected_ref_value => {
                if let Err(e) = conn.del::<_, ()>(&ref_key).await {
                    warn!("upload-token: failed to delete ref key '{}': {e}", ref_key);
                }
            }
            Ok(_) => {}
            Err(e) => {
                warn!("upload-token: failed to read ref key '{}': {e}", ref_key);
            }
        }
    }

    // ── 反序列化辅助 ──────────────────────────────────────────────────────────

    pub(super) fn parse_session(raw: &str) -> crate::dao::result::FileManagerResult<UploadSessionData> {
        use lsys_core::fluent_message;
        use crate::dao::result::FileManagerError;
        serde_json::from_str(raw)
            .map_err(|e| FileManagerError::Message(fluent_message!("upload-token-parse-error", e)))
    }
}
