//! 上传超时清理任务
//!
//! 定期扫描 `lst_file` 表中处于 `Unfinished` 状态且超过允许上传时限的记录，
//! 将其标记为 `Failed`，以便后续 `delete_file` 可以正常清理。
//!
//! ## 设计决策
//!
//! * 超时阈值默认 **24 小时**（可通过 `file_upload_unfinished_timeout` 配置项调整），
//!   故意选长不选短，避免因网络慢、大文件上传等情况被误判为超时。
//! * 此任务**只做状态变更**（`Unfinished → Failed`），不负责物理文件清理；
//!   物理清理由调用者通过 `delete_file` 完成。
//! * 同一时刻仅一个节点执行（通过 Redis 分布式锁，参见 `TimeOutTask`）。

use std::future::Future;
use std::pin::Pin;

use lsys_core::db::TableMeta;
use lsys_core::timeout_task::{TimeOutTaskExec, TimeOutTaskExecutor, TimeOutTaskNextTime};
use lsys_core::utils::now_time;
use sqlx::{MySql, Pool};
use tracing::{info, warn};

use crate::model::{FileModel, FileStatus};

/// 上传超时扫描任务
///
/// 每次被唤醒时，以批次（100 条/次）扫描所有超时未完成的文件记录，
/// 将其状态从 `Unfinished` 更新为 `Failed`。
pub struct FileUnfinishedTimeoutTask {
    db: Pool<MySql>,
    /// 与 `FileConfig::upload_max_duration` 共享：超过此秒数仍 Unfinished 则判定为超时
    upload_max_duration: u64,
}

impl FileUnfinishedTimeoutTask {
    pub fn new(db: Pool<MySql>, upload_max_duration: u64) -> Self {
        Self {
            db,
            upload_max_duration,
        }
    }
}

#[async_trait::async_trait]
impl TimeOutTaskExec for FileUnfinishedTimeoutTask {
    async fn exec(
        &self,
        max_lock_time: usize,
        mut expire_call: impl FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send,
    ) -> Result<(), String> {
        let ntime = now_time().unwrap_or_default();
        // 超时截止时间点：add_time <= deadline 的记录已超时
        let deadline = ntime.saturating_sub(self.upload_max_duration);
        let mut start_id = 0u64;
        let mut runtime = ntime;
        let mut total_updated = 0u64;

        info!(
            "FileUnfinishedTimeoutTask: start scan, deadline={} (upload_max_duration={}s)",
            deadline, self.upload_max_duration
        );

        loop {
            // 批量查询超时 Unfinished 记录（仅限上传来源，排除下载任务滞留）
            let rows: Vec<(u64,)> = sqlx::query_as(&format!(
                "SELECT id FROM {} WHERE id > ? AND status = ? AND add_time <= ? ORDER BY id ASC LIMIT 100",
                FileModel::table_name()
            ))
            .bind(start_id)
            .bind(FileStatus::Unfinished as i8)
            .bind(deadline)
            .fetch_all(&self.db)
            .await
            .map_err(|e| format!("FileUnfinishedTimeoutTask: query failed: {}", e))?;

            if rows.is_empty() {
                break;
            }

            let ids: Vec<u64> = rows.iter().map(|(id,)| *id).collect();
            let max_id = match ids.last() {
                Some(&id) => id,
                None => break, // rows was non-empty but ids somehow empty, stop safely
            };

            if ids.is_empty() {
                break;
            }

            // 批量更新为 Failed
            let placeholders = ids
                .iter()
                .map(|_| "?")
                .collect::<Vec<_>>()
                .join(", ");
            let sql = format!(
                "UPDATE {} SET status = ? WHERE id IN ({}) AND status = ?",
                FileModel::table_name(),
                placeholders
            );

            let mut query = sqlx::query(&sql).bind(FileStatus::Failed as i8);
            for id in &ids {
                query = query.bind(id);
            }
            query = query.bind(FileStatus::Unfinished as i8);

            match query.execute(&self.db).await {
                Ok(result) => {
                    let affected = result.rows_affected();
                    total_updated += affected;
                    info!(
                        "FileUnfinishedTimeoutTask: updated {} rows to Failed (ids {}..={})",
                        affected,
                        ids[0],
                        max_id
                    );
                }
                Err(e) => {
                    warn!(
                        "FileUnfinishedTimeoutTask: update failed for ids {}..={}: {}",
                        ids[0], max_id, e
                    );
                    // 跳过这批，继续下一批，避免单批错误阻塞整个任务
                }
            }

            start_id = max_id;

            // 超时保护：若本次 exec 已运行超过 max_lock_time，提前返回
            let now = now_time().unwrap_or_default();
            if now.saturating_sub(runtime) >= max_lock_time as u64 {
                return Err(format!(
                    "FileUnfinishedTimeoutTask: timeout [elapsed={}s, limit={}s, total_updated={}]",
                    now.saturating_sub(ntime),
                    max_lock_time,
                    total_updated,
                ));
            }

            // 若已超过一半 max_lock_time，续租 Redis 锁
            if now.saturating_sub(runtime) * 2 >= max_lock_time as u64 {
                expire_call().await;
            }
            runtime = now;
        }

        info!(
            "FileUnfinishedTimeoutTask: completed, total_updated={}",
            total_updated
        );
        Ok(())
    }
}

#[async_trait::async_trait]
impl TimeOutTaskNextTime for FileUnfinishedTimeoutTask {
    async fn next_time(&self, _max_lock_time: usize) -> Result<Option<u64>, String> {
        // 查询最早超时的 Unfinished 上传记录的触发时间 = add_time + upload_unfinished_timeout
        let row: Option<(u64,)> = sqlx::query_as(&format!(
            "SELECT add_time FROM {} WHERE status = ? ORDER BY add_time ASC LIMIT 1",
            FileModel::table_name()
        ))
        .bind(FileStatus::Unfinished as i8)
        .fetch_optional(&self.db)
        .await
        .map_err(|e| format!("FileUnfinishedTimeoutTask: next_time query failed: {}", e))?;

        match row {
            Some((add_time,)) => {
                let trigger = add_time.saturating_add(self.upload_max_duration);
                info!(
                    "FileUnfinishedTimeoutTask: next trigger time={}",
                    trigger
                );
                Ok(Some(trigger))
            }
            None => {
                info!("FileUnfinishedTimeoutTask: no pending Unfinished records");
                Ok(None)
            }
        }
    }
}

pub struct FileUnfinishedTimeoutTaskExecutor;

impl TimeOutTaskExecutor for FileUnfinishedTimeoutTaskExecutor {
    type Exec = FileUnfinishedTimeoutTask;
    type NextTime = FileUnfinishedTimeoutTask;
}
