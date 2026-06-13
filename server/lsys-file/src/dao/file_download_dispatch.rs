// 基于 lsys-core task_dispatch 实现多节点文件下载任务派发
//
// 功能特性:
// - 基于 Redis 的分布式任务派发
// - 支持多节点并行下载，避免重复处理
// - 支持分片下载的多节点处理
// - 任务超时检测和重试机制
// - 使用 WaitNotify 实现跨节点下载完成通知
//
// 使用方式:
// 1. 启用 redis 特性
// 2. 创建 FileDownloadDispatchManager
// 3. 在每个节点上调用 listen() 启动监听
// 4. 调用 notify() 触发下载任务
// 5. 使用 wait_download() 等待下载完成
//
// 详细文档请参考: DOWNLOAD_DISPATCH.md
// 参考实现: lsys-app-sender/src/dao/sender_task.rs

use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;

use async_trait::async_trait;
use lsys_core::app_core::AppCore;
use lsys_core::db::{QueryBuilderExt, TableMeta};
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::listen_notify::{WaitItem, WaitNotify, WaitNotifyResult};
use lsys_core::task_dispatch::{
    TaskAcquisition, TaskData, TaskDispatch, TaskDispatchConfig, TaskExecutor, TaskItem,
    TaskNotify, TaskNotifyConfig, TaskRecord,
};
use redis::{FromRedisValue, ParsingError, ToRedisArgs};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tokio::sync::oneshot::Receiver;
use tracing::{info, warn};

use super::file_helpers::FileHelper;
use crate::model::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct DownloadWaitItem {
    pub file_ref_id: u64,
}

impl WaitItem for DownloadWaitItem {
    fn eq(&self, other: &Self) -> bool {
        self.file_ref_id == other.file_ref_id
    }
}

/// 下载等待通知管理器
pub struct DownloadWaitNotify(WaitNotify<DownloadWaitItem>);

impl std::ops::Deref for DownloadWaitNotify {
    type Target = WaitNotify<DownloadWaitItem>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DownloadWaitNotify {
    pub fn new(redis: deadpool_redis::Pool, app_core: Arc<AppCore>, clear_timeout: u8) -> Self {
        DownloadWaitNotify(WaitNotify::<DownloadWaitItem>::new(
            "file-download-wait",
            redis,
            app_core,
            clear_timeout,
        ))
    }

    /// 等待下载完成
    pub async fn wait_download(&self, file_ref_id: u64) -> Receiver<WaitNotifyResult> {
        self.wait(DownloadWaitItem {
            file_ref_id,
        })
        .await
    }

    /// 通知下载完成
    pub async fn notify_download(&self, host: &str, file_ref_id: u64, result: WaitNotifyResult) {
        if host.is_empty() {
            return;
        }

        if let Err(e) = self
            .0
            .notify(host, DownloadWaitItem { file_ref_id }, result)
            .await
        {
            warn!(
                "notify_download failed for file_ref_id={}: {}",
                file_ref_id,
                e.to_fluent_message().default_format()
            );
        }
    }
}

/// 下载任务唯一标识: file_id + chunk_index
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DownloadTaskId {
    pub file_id: u64,
    pub chunk_index: u32,
}

impl Display for DownloadTaskId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}_{}", self.file_id, self.chunk_index)
    }
}

impl FromRedisValue for DownloadTaskId {
    fn from_redis_value(val: redis::Value) -> Result<Self, ParsingError> {
        match val {
            redis::Value::BulkString(bytes) => {
                let valstr = std::str::from_utf8(&bytes)
                    .map_err(|e| ParsingError::from(format!("UTF-8 decode error: {}", e)))?;
                serde_json::from_str::<DownloadTaskId>(valstr)
                    .map_err(|e| ParsingError::from(format!("JSON parse error: {}", e)))
            }
            redis::Value::SimpleString(s) => serde_json::from_str::<DownloadTaskId>(&s)
                .map_err(|e| ParsingError::from(format!("JSON parse error: {}", e))),
            other => Err(ParsingError::from(format!(
                "Response type not string compatible: {:?}",
                other
            ))),
        }
    }
}

impl ToRedisArgs for DownloadTaskId {
    fn write_redis_args<W>(&self, out: &mut W)
    where
        W: ?Sized + redis::RedisWrite,
    {
        out.write_arg(serde_json::to_string(self).unwrap_or_default().as_bytes())
    }
}

impl redis::ToSingleRedisArg for DownloadTaskId {}

/// 下载任务项 (单个分片)
#[derive(Debug, Clone)]
pub struct DownloadTaskItem {
    pub file_id: u64,
    pub chunk_index: u32,
    pub file_ref_id: u64,
    pub trigger_host: String,
}

impl TaskItem<DownloadTaskId> for DownloadTaskItem {
    fn to_task_pk(&self) -> DownloadTaskId {
        DownloadTaskId {
            file_id: self.file_id,
            chunk_index: self.chunk_index,
        }
    }
}

/// 下载任务获取器
pub struct DownloadTaskAcquisition {
    helper: Arc<FileHelper>,
}

impl DownloadTaskAcquisition {
    pub fn new(helper: Arc<FileHelper>) -> Self {
        Self { helper }
    }
}

#[async_trait]
impl TaskAcquisition<DownloadTaskId, DownloadTaskItem> for DownloadTaskAcquisition {
    async fn read_exec_task(
        &self,
        tasking_record: &HashMap<DownloadTaskId, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<DownloadTaskId, DownloadTaskItem>, String> {
        let db = &self.helper.db;

        // 构建排除条件: 已在执行中的任务
        let exclude_tasks: Vec<(u64, u32)> = tasking_record
            .keys()
            .map(|id| (id.file_id, id.chunk_index))
            .collect();

        info!(
            "read_exec_task: limit={}, running_tasks={}, excluding=[{}]",
            limit,
            tasking_record.len(),
            exclude_tasks
                .iter()
                .map(|(f, c)| format!("{}_{}", f, c))
                .collect::<Vec<_>>()
                .join(", ")
        );

        // 查询待下载的分片
        // 策略：
        // 1. 对于分片文件: 查询 file_local_chunk 中未完成的分片
        // 2. 对于非分片文件: 查询 file_local 记录，chunk_index 使用 0
        //
        // 优化：只查询必要的字段 (file_id, chunk_index, file_ref_id, trigger_host)
        let mut qb = sqlx::QueryBuilder::new(format!(
            "SELECT fu.id as file_ref_id, fu.file_id, fu.trigger_host, \
                    COALESCE(flc.chunk_index, 0) as chunk_index \
             FROM {} fu \
             INNER JOIN {} fl ON fu.file_id = fl.file_id \
             INNER JOIN {} f ON fu.file_id = f.id \
             LEFT JOIN {} flc ON fu.file_id = flc.file_id AND flc.status = {}",
            FileRefModel::table_name(),
            FileLocalModel::table_name(),
            FileModel::table_name(),
            FileLocalChunkModel::table_name(),
            FileChunkStatus::Unfinished as i8,
        ));

        // 只查询源URL为HTTP的下载任务
        qb.push_where().push(" fu.source_url != ''");

        // 只查询有效的文件引用（未删除）
        qb.push_and().push(" fu.status = ");
        qb.push_bind(FileUserStatus::Normal as i8);

        // 只查询未完成状态的文件，防止失败文件（file.status=Failed）被无限重试
        qb.push_and().push(" f.status = ");
        qb.push_bind(FileStatus::Unfinished as i8);

        // 分片文件必须有未完成的chunk，或者是非分片文件且未下载
        // 注意：括号确保 AND 优先级正确，避免 OR 分支绕过上方的 source_url/status 过滤
        qb.push_and().push(" ((fl.file_chunk_total > 0 AND flc.id IS NOT NULL) OR (fl.file_chunk_total = 0 AND fl.local_path = ''))");

        // 添加排除正在执行的任务
        if !exclude_tasks.is_empty() {
            qb.push_and().push(" NOT (");
            for (idx, (file_id, chunk_index)) in exclude_tasks.iter().enumerate() {
                if idx > 0 {
                    qb.push(" OR ");
                }
                qb.push(" (fu.file_id = ");
                qb.push_bind(*file_id);
                qb.push(" AND COALESCE(flc.chunk_index, 0) = ");
                qb.push_bind(*chunk_index);
                qb.push(")");
            }
            qb.push(")");
        }

        qb.push(format!(" ORDER BY fu.id ASC, COALESCE(flc.chunk_index, 0) ASC LIMIT {}", limit));

        let rows = qb
            .build()
            .fetch_all(db)
            .await
            .map_err(|e| {
                warn!("read_exec_task: DB query failed: {}", e);
                format!("query download tasks failed: {}", e)
            })?;

        info!(
            "read_exec_task: DB returned {} rows",
            rows.len()
        );

        let mut result = Vec::new();
        for row in rows {
            let file_ref_id: u64 = row.try_get("file_ref_id").map_err(|e| e.to_string())?;
            let file_id: u64 = row.try_get("file_id").map_err(|e| e.to_string())?;
            // COALESCE(nullable INT UNSIGNED, 0) is promoted to BIGINT by MySQL at runtime,
            // so decode as i64 then cast to u32 at the DB boundary.
            let chunk_index: u32 = row
                .try_get::<i64, _>("chunk_index")
                .map_err(|e| e.to_string())? as u32;
            let trigger_host: String = row.try_get("trigger_host").map_err(|e| e.to_string())?;

            result.push(DownloadTaskItem {
                file_id,
                chunk_index,
                file_ref_id,
                trigger_host,
            });

            if result.len() >= limit {
                break;
            }
        }

        let has_next = result.len() >= limit;

        if result.is_empty() {
            info!("read_exec_task: no pending download tasks found");
        } else {
            info!(
                "read_exec_task: dispatching {} tasks, has_next={}; items=[{}]",
                result.len(),
                has_next,
                result
                    .iter()
                    .map(|t| format!(
                        "file_id={} chunk={} ref={}",
                        t.file_id, t.chunk_index, t.file_ref_id
                    ))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        Ok(TaskRecord::new(result, has_next))
    }
}

/// 下载任务执行器
pub struct DownloadTaskExecutorImpl {
    helper: Arc<FileHelper>,
    wait_notify: Arc<DownloadWaitNotify>,
}

impl DownloadTaskExecutorImpl {
    pub fn new(helper: Arc<FileHelper>, wait_notify: Arc<DownloadWaitNotify>) -> Self {
        Self {
            helper,
            wait_notify,
        }
    }
}

#[async_trait]
impl TaskExecutor<DownloadTaskId, DownloadTaskItem> for DownloadTaskExecutorImpl {
    async fn exec(&self, task: DownloadTaskItem) -> Result<(), String> {
        info!(
            "executing download task: file_id={}, chunk_index={}",
            task.file_id, task.chunk_index
        );

        // 直接调用 FileDownloadCore::execute_download
        let result = super::file_download::FileDownloadCore::execute_download(
            &self.helper,
            task.file_ref_id,
            task.chunk_index,
        )
        .await;

        match result {
            Ok(download_result) => {
                match download_result {
                    super::file_download::DownloadResult::Completed => {
                        // 文件下载完成，通知触发主机
                        info!(
                            "file download completed: file_id={}, notifying trigger_host={}, file_ref_id={}",
                            task.file_id, task.trigger_host, task.file_ref_id
                        );
                        self.wait_notify
                            .notify_download(&task.trigger_host, task.file_ref_id, Ok(true))
                            .await;
                        Ok(())
                    }
                    super::file_download::DownloadResult::ChunkCompleted => {
                        // 分片下载完成，但文件还有其他分片未完成，不通知
                        info!(
                            "chunk download completed: file_id={}, chunk_index={}, waiting for others",
                            task.file_id, task.chunk_index
                        );
                        Ok(())
                    }
                    super::file_download::DownloadResult::Failed(msg) => {
                        // 下载失败，通知触发主机
                        warn!(
                            "file download failed: file_id={}, chunk_index={}, error={}",
                            task.file_id, task.chunk_index, msg
                        );
                        self.wait_notify
                            .notify_download(&task.trigger_host, task.file_ref_id, Err(msg.clone()))
                            .await;
                        Err(msg)
                    }
                }
            }
            Err(e) => {
                let err_msg = format!(
                    "download task error: {}",
                    e.to_fluent_message().default_format()
                );
                warn!("{}", err_msg);
                self.wait_notify
                    .notify_download(&task.trigger_host, task.file_ref_id, Err(err_msg.clone()))
                    .await;
                Err(err_msg)
            }
        }
    }
}

/// 文件下载任务派发管理器 (多节点版本)
pub struct FileDownloadDispatchManager {
    pub task_dispatch: Arc<TaskDispatch<DownloadTaskId, DownloadTaskItem>>,
    pub task_notify: Arc<TaskNotify>,
    pub wait_notify: Arc<DownloadWaitNotify>,
}

impl FileDownloadDispatchManager {
    pub fn new(
        redis: deadpool_redis::Pool,
        app_core: Arc<AppCore>,
        download_config: &super::file_config::FileDownloadConfig,
    ) -> Self {
        let notify_config = Arc::new(TaskNotifyConfig::new("file-download"));
        let task_notify = Arc::new(TaskNotify::new(redis.clone(), notify_config.clone()));
        let dispatch_config = Arc::new(TaskDispatchConfig::new(
            notify_config,
            download_config.task_timeout,
            download_config.is_timeout_check,
            download_config.task_size,
        ));

        let task_dispatch = Arc::new(TaskDispatch::new(
            redis.clone(),
            task_notify.clone(),
            dispatch_config,
        ));

        let wait_notify = Arc::new(DownloadWaitNotify::new(
            redis,
            app_core,
            download_config.wait_timeout,
        ));

        Self {
            task_dispatch,
            task_notify,
            wait_notify,
        }
    }
}
