use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use lsys_core::db::TableMeta;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::timeout_task::{TimeOutTaskExec, TimeOutTaskExecutor, TimeOutTaskNextTime};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool};
use tracing::{info, warn};

use crate::model::{FileRefModel, FileUserStatus};

use super::file_ops::FileOps;

/// 文件过期自动删除任务
///
/// 定期扫描 lst_file_ref 表中 expire_time 已到期的记录，
/// 调用 FileOps::delete_file 进行删除。
pub struct FileExpirationTask {
    db: Pool<MySql>,
    file_ops: Arc<FileOps>,
}

impl FileExpirationTask {
    pub fn new(db: Pool<MySql>, file_ops: Arc<FileOps>) -> Self {
        Self { db, file_ops }
    }

    /// 删除单个过期文件
    async fn delete_expired_file(&self, file_ref: &FileRefModel) -> Result<(), String> {
        info!(
            "Deleting expired file: file_ref_id={}, file_id={}, user_id={}, expire_time={}",
            file_ref.id, file_ref.file_id, file_ref.user_id, file_ref.expire_time
        );

        let ctx = self.file_ops.create_context(file_ref);

        if let Err(e) = self.file_ops.delete_file(ctx, None).await {
            warn!(
                "Failed to delete expired file: file_ref_id={}, error: {}",
                file_ref.id,
                e.to_fluent_message().default_format()
            );
            return Err(format!(
                "Failed to delete file_ref_id={}: {}",
                file_ref.id,
                e.to_fluent_message().default_format()
            ));
        }

        info!(
            "Successfully deleted expired file: file_ref_id={}",
            file_ref.id
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl TimeOutTaskExec for FileExpirationTask {
    async fn exec(
        &self,
        max_lock_time: usize,
        mut expire_call: impl FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send,
    ) -> Result<(), String> {
        let ntime = now_time().unwrap_or_default();
        let mut runtime = ntime;
        let mut start_id = 0u64;

        info!("FileExpirationTask: Starting execution at time={}", ntime);

        loop {
            // 查询已过期的文件（expire_time > 0 且 <= 当前时间，status 为 Normal）
            let expired_files = sqlx::query_as::<_, FileRefModel>(&format!(
                "SELECT * FROM {} WHERE id > ? AND status = ? AND expire_time > 0 AND expire_time <= ? ORDER BY id ASC LIMIT 100",
                FileRefModel::table_name()
            ))
            .bind(start_id)
            .bind(FileUserStatus::Normal as i8)
            .bind(ntime)
            .fetch_all(&self.db)
            .await
            .map_err(|e| format!("Failed to query expired files: {}", e))?;

            if expired_files.is_empty() {
                info!("FileExpirationTask: No more expired files found");
                break;
            }

            info!(
                "FileExpirationTask: Found {} expired files to delete",
                expired_files.len()
            );

            for file_ref in expired_files {
                start_id = file_ref.id;

                // 删除过期文件
                if let Err(e) = self.delete_expired_file(&file_ref).await {
                    warn!("FileExpirationTask: {}", e);
                    // 继续处理下一个文件，不中断整个任务
                }

                // 检查是否超时
                let last_now_time = now_time().unwrap_or_default();
                if (last_now_time - runtime) > (max_lock_time as u64) {
                    return Err(format!(
                        "FileExpirationTask timeout [last run time:{}, start time:{}]",
                        last_now_time, runtime
                    ));
                }

                // 如果运行时间超过一半，延长锁定时间
                if (last_now_time - runtime) * 2 > (max_lock_time as u64) {
                    expire_call().await;
                }

                runtime = last_now_time;
            }
        }

        info!("FileExpirationTask: Execution completed successfully");
        Ok(())
    }
}

#[async_trait::async_trait]
impl TimeOutTaskNextTime for FileExpirationTask {
    async fn next_time(&self, max_lock_time: usize) -> Result<Option<u64>, String> {
        let ntime = now_time().unwrap_or_default();

        // 查询最近一个将要过期的文件时间
        let next_expire_time = sqlx::query_scalar::<_, u64>(&format!(
            "SELECT expire_time FROM {} WHERE status = ? AND expire_time > 0 AND expire_time <= ? ORDER BY expire_time ASC LIMIT 1",
            FileRefModel::table_name()
        ))
        .bind(FileUserStatus::Normal as i8)
        .bind(ntime + max_lock_time as u64)
        .fetch_one(&self.db)
        .await;

        match next_expire_time {
            Ok(time) => {
                info!("FileExpirationTask: Next expiration time={}", time);
                Ok(Some(time))
            }
            Err(sqlx::Error::RowNotFound) => {
                info!("FileExpirationTask: No upcoming expirations found");
                Ok(None)
            }
            Err(err) => Err(format!("Failed to query next expiration time: {}", err)),
        }
    }
}

#[async_trait::async_trait]
impl TimeOutTaskExecutor for FileExpirationTask {
    type Exec = Self;
    type NextTime = Self;
}
