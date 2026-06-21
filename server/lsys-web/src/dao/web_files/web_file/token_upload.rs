//! 令牌上传完成逻辑（service 与 rest 两类场景共用）
//!
//! 同时支持单文件令牌（`session_id = None`）和分片令牌（`session_id = Some`）。
//! - 单文件：文件进入终态时调用 `remove_token` 销毁令牌并清理反向索引。
//! - 分片：分片进入终态时：所有分片完成（Normal）或失败（Failed）→ 调用 `remove_session` 批量清理；
//!   否则仅销毁当前分片令牌。

use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::RequestEnv;
use lsys_file::model::FileStatus;
use serde_json::{json, Value};
use tracing::warn;

use super::WebFile;
use crate::dao::result::WebResult;

impl WebFile {
    /// 通过上传令牌完成一个分片的写入。
    ///
    /// 支持单文件令牌和分片会话令牌：
    /// - 解析令牌获取 `file_ref_id / user_id / app_id`（以及可选的 `session_id / part_number`）。
    /// - 写入分片数据，文件进入终态时清理令牌（单文件）或整个会话（分片）。
    /// - 当 `app_id != 0` 且文件进入 `Normal` 终态时，触发上传完成回调。
    pub async fn finish_token_chunk_upload(
        &self,
        token: &str,
        chunk_index: u32,
        data: &[u8],
        env_data: Option<&RequestEnv>,
    ) -> WebResult<Value> {
        let token_data = self.upload_token.resolve_token(token).await?;
        let file_ref_id = token_data.file_ref_id;

        let mut handle = self
            .file_dao
            .get_upload_handle_by_file_ref_id(file_ref_id, chunk_index)
            .await?;
        let file_id = handle.file.id;

        match self.file_dao.write_file(&mut handle, data).await {
            Ok(_) => {
                let completed_file = self.file_dao.complete_upload(handle, env_data).await?;

                let is_terminal = completed_file.status == FileStatus::Normal as i8
                    || completed_file.status == FileStatus::Failed as i8;

                if is_terminal {
                    self.cleanup_token_on_terminal(token, &token_data).await;
                }

                let file_url = self
                    .file_dao
                    .data_dao()
                    .get_file_url(&completed_file)
                    .await
                    .ok()
                    .flatten();

                let payload = json!({
                    "id": file_ref_id,
                    "file_id": completed_file.id,
                    "chunk_index": chunk_index,
                    "status": completed_file.status,
                    "file_md5": completed_file.file_md5,
                    "file_name": completed_file.origin_name,
                    "file_size": completed_file.file_size,
                    "content_type": completed_file.content_type,
                    "storage_type": completed_file.storage_type,
                    "file_url": file_url,
                });

                // 文件 Normal 终态 + 归属应用时触发回调
                if completed_file.status == FileStatus::Normal as i8 && token_data.app_id != 0
                    && let Err(err) = self
                        .file_notify_sender
                        .send(
                            token_data.app_id,
                            &file_ref_id.to_string(),
                            &payload.to_string(),
                        )
                        .await
                {
                    warn!(
                        "file upload notify send fail: {}",
                        err.to_fluent_message().default_format()
                    );
                }

                Ok(payload)
            }
            Err(e) => {
                let file_local = self
                    .file_dao
                    .helper()
                    .find_file_local_by_file_id(file_id)
                    .await
                    .ok()
                    .flatten();

                if let Err(err) = self.file_dao.fail_upload(handle, env_data).await {
                    warn!("fail_upload error:{}", err.to_fluent_message().default_format());
                }

                // 单分片文件失败时销毁令牌
                if let Some(local) = file_local
                    && local.file_chunk_total <= 1
                {
                    self.cleanup_token_on_terminal(token, &token_data).await;
                }
                Err(e.into())
            }
        }
    }

    /// 终态清理：单文件令牌直接销毁；分片令牌销毁后若归属会话则清理整个会话。
    async fn cleanup_token_on_terminal(
        &self,
        token: &str,
        token_data: &lsys_file_manager::dao::UploadTokenData,
    ) {
        if let Some(session_id) = &token_data.session_id {
            // 先移除当前分片令牌，再清理会话
            if let Err(e) = self.upload_token.remove_token(token).await {
                warn!(
                    "upload-token: failed to remove part token '{}': {}",
                    token,
                    e.to_fluent_message().default_format()
                );
            }
            if let Err(e) = self.upload_token.remove_session(session_id).await {
                warn!(
                    "upload-token: failed to remove session '{}': {}",
                    session_id,
                    e.to_fluent_message().default_format()
                );
            }
        } else {
            if let Err(e) = self.upload_token.remove_token(token).await {
                warn!(
                    "upload-token: failed to remove token '{}': {}",
                    token,
                    e.to_fluent_message().default_format()
                );
            }
        }
    }
}

