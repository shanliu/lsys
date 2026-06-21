// 导出任务调度与执行

use std::collections::HashMap;
use std::sync::Arc;

use lsys_core::fluents::FluentMgr;

use lsys_core::db::{QueryBuilderExt, TableMeta, Update};

use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::now_time;
use lsys_file::dao::{LocalFileMode, LocalFileSource};
use sqlx::{MySql, Pool};
use tokio::sync::{Semaphore, mpsc};
use tracing::{Instrument, error, info, warn};

use crate::dao::result::FileManagerResult;
use crate::model::*;

use super::ExportTaskConfig;
use super::exporter::Exporter;

/// 导出任务调度器
///
/// 独立结构，仅用于执行后台调度循环。
/// 通过 Arc 共享 ExportTask 的部分字段，并独占 trigger_rx。
pub struct ExportTaskDispatcher {
    /// 数据库连接池（clone Pool，内部是 Arc）
    pub(crate) db: Pool<MySql>,
    /// 文件 DAO（Arc clone，共享引用）
    pub(crate) file_dao: Arc<lsys_file::dao::FileDao>,
    /// 配置（clone 值）
    pub(crate) config: ExportTaskConfig,
    /// 导出器注册表（Arc clone，共享引用）
    pub(crate) exporters: Arc<HashMap<String, Box<dyn Exporter<crate::dao::FileManagerError>>>>,
    /// 并发控制信号量（Arc clone，共享引用）
    pub(crate) semaphore: Arc<Semaphore>,
    /// 触发信号接收端（独占所有权）
    pub(crate) trigger_rx: mpsc::Receiver<()>,
    /// 多语言管理器，供 exporter 解析 locale
    pub(crate) fluent_mgr: Arc<FluentMgr>,
}


impl ExportTaskDispatcher {
    /// 后台调度循环（应在应用启动时 spawn 一次）
    ///
    /// 持续监听 trigger 信号，收到后从 DB 拉取 Pending 记录并投递到执行池。
    pub async fn dispatch_loop(mut self, cancel_token: tokio_util::sync::CancellationToken) {
        loop {
            // 等待触发信号
            tokio::select! {
                msg = self.trigger_rx.recv() => {
                    if msg.is_none() {
                        info!("export_task: trigger channel closed, stopping dispatch loop");
                        break;
                    }
                }
                _ = cancel_token.cancelled() => {
                    info!("export_task: cancelled, stopping dispatch loop");
                    break;
                }
            }

            // 收到信号后，先把 channel 里积压的信号全部清空
            while self.trigger_rx.try_recv().is_ok() {}

            // 每次调度创建独立 span，记录单次执行的起点、过程、结束
            let task_id = lsys_core::utils::rand_str(lsys_core::utils::RandType::LowerHex, 8);
            // 从 DB 拉取 Pending 记录
            if let Err(e) = self.dispatch_pending()
                .instrument(tracing::info_span!(
                    "background_task",
                    task = "export-task-dispatch",
                    task_id = task_id
                ))
                .await
            {
                error!(
                    "export_task: dispatch_pending error: {}",
                    e.to_fluent_message().default_format()
                );
            }
        }
    }

    /// 逐条拉取 Pending 记录并投递到执行池
    ///
    /// 以 id 游标递增扫描，每次等 JoinSet 有空位（semaphore）后再读取下一条，
    /// 直到查不到更多 Pending 记录才退出；最后等待全部任务完成并执行超时检测。
    async fn dispatch_pending(&self) -> FileManagerResult<()> {
        let mut last_id: u64 = 0;
        let mut join_set = tokio::task::JoinSet::new();

        loop {
            // 先回收已完成的任务（非阻塞），释放 semaphore 名额
            while let Some(result) = join_set.try_join_next() {
                if let Err(e) = result {
                    error!("export_task: spawned task panicked: {}", e);
                }
            }

            // 等待并发名额空闲，保证不会超出 limit_branch
            let permit = match Arc::clone(&self.semaphore).acquire_owned().await {
                Ok(p) => p,
                Err(e) => {
                    error!("export_task: semaphore closed: {}", e);
                    break;
                }
            };

            // 读取下一条 Pending 记录（id > last_id）
            let sql = format!(
                "SELECT * FROM {} WHERE status=? AND id>? ORDER BY id ASC LIMIT 1",
                ExportTaskModel::table_name()
            );

            let record = match sqlx::query_as::<_, ExportTaskModel>(&sql)
                .bind(ExportTaskStatus::Pending as i8)
                .bind(last_id)
                .fetch_optional(&self.db)
                .await?
            {
                Some(r) => r,
                None => break, // 没有更多 Pending 记录，退出
            };

            last_id = record.id;

            // CAS 更新状态: Pending → Running（确保只有一个实例处理）
            let now = now_time().unwrap_or_default();
            let affected = Update::<_, ExportTaskModel>::new()
                .set(ExportTaskModel::STATUS, ExportTaskStatus::Running as i8)
                .set(ExportTaskModel::CHANGE_TIME, now)
                .execute(&self.db, |qb| {
                    qb.push_where().field_eq("id", record.id);
                    qb.push_and()
                        .field_eq("status", ExportTaskStatus::Pending as i8);
                })
                .await?
                .rows_affected();

            if affected == 0 {
                warn!(
                    "export_task: record id={} already taken by another worker, skip",
                    record.id
                );
                // permit 在此处 drop，名额归还
                continue;
            }

            info!(
                "export_task: record id={} marked as Running, dispatching",
                record.id
            );

            let db = self.db.clone();
            let file_dao = self.file_dao.clone();
            let exporters = Arc::clone(&self.exporters);
            let fluent_mgr = Arc::clone(&self.fluent_mgr);

            let record_id = record.id;
            join_set.spawn(
                async move {
                    let _permit = permit; // 持有 permit 直到任务结束

                    Self::execute_task(&db, &file_dao, &exporters, fluent_mgr, record).await;
                }
                .instrument(tracing::info_span!(
                    "background_task",
                    task = "export-task-execute",
                    task_id = record_id
                )),
            );
        }

        // 等待所有剩余任务执行完成
        while let Some(result) = join_set.join_next().await {
            if let Err(e) = result {
                error!("export_task: spawned task panicked: {}", e);
            }
        }

        // 全部完成后，执行超时检测
        let timeout_secs = self.config.timeout_secs;
        if timeout_secs > 0
            && let Err(e) = self.mark_timeout_tasks(timeout_secs).await
        {
            error!(
                "export_task: mark_timeout_tasks error: {}",
                e.to_fluent_message().default_format()
            );
        }

        Ok(())
    }

    /// 执行单个导出任务
    async fn execute_task(
        db: &sqlx::Pool<sqlx::MySql>,
        file_dao: &Arc<lsys_file::dao::FileDao>,
        exporters: &HashMap<
            String,
            Box<dyn super::exporter::Exporter<crate::dao::FileManagerError>>,
        >,
        fluent_mgr: Arc<FluentMgr>,
        record: ExportTaskModel,
    ) {
        let task_id = record.id;

        // 1. 查找 exporter
        let exporter: &dyn super::exporter::Exporter<crate::dao::FileManagerError> =
            match exporters.get(&record.export_type) {
                Some(exp) => exp.as_ref(),
                None => {
                    let err_msg = format!("export_type '{}' not registered", record.export_type);
                    error!("export_task: task id={} error: {}", task_id, err_msg);
                    Self::mark_failed(db, task_id, &err_msg).await;
                    return;
                }
            };

        // 2. 解析 export_params
        let params: serde_json::Value = match serde_json::from_str(&record.export_params) {
            Ok(v) => v,
            Err(e) => {
                let err_msg = format!("failed to parse export_params: {}", e);
                error!("export_task: task id={} error: {}", task_id, err_msg);
                Self::mark_failed(db, task_id, &err_msg).await;
                return;
            }
        };

        // export 之后还需要这几个字段，提前取出，record 按值移入 export
        let export_type = record.export_type.clone();
        let app_id = record.app_id;
        let add_user_id = record.add_user_id;
        let record_user_id = record.user_id;
        let request_id = record.request_id.clone();
        let lang = if record.lang.is_empty() { None } else { Some(record.lang.clone()) };

        // 3. 执行导出（record 按值传入，future 内部拥有所有权）
        // exporter 返回的已经是 FileManagerError
        let result: Result<std::path::PathBuf, crate::dao::FileManagerError> =
            exporter.export(record, params, lang, fluent_mgr).await;

        match result {
            Err(e) => {
                let err_msg = e.to_fluent_message().default_format();
                error!(
                    "export_task: task id={} export failed: {}",
                    task_id, err_msg
                );
                Self::mark_failed(db, task_id, &err_msg).await;
            }
            Ok(file_path) => {
                // 4. 将文件 Move 到 lsys-file 存储，打 TAG
                let path_str: String = file_path.to_string_lossy().to_string();
                let tag_export = format!("export_id_{}", task_id);
                let tag_type = format!("export_type_{}", export_type);
                let tag_request_id = format!("request_{}", request_id);
                let tag_names: Vec<&str> = vec![&tag_export, &tag_type, &tag_request_id];

                match file_dao
                    .create_from_local_file(
                        &path_str,
                        record_user_id,
                        add_user_id,
                        app_id,
                        lsys_file::model::FileModel::STORAGE_TYPE_LOCAL_PRIVATE,
                        None,
                        LocalFileMode::Move,
                        LocalFileSource::Plaintext,
                        false,
                        &tag_names,
                        None, // expire_time
                        None,
                    )
                    .await
                {
                    Ok((file_model, _file_ref)) => {
                        info!(
                            "export_task: task id={} file created: file_id={}, file_name={}",
                            task_id, file_model.id, file_model.origin_name
                        );
                        Self::mark_success(db, task_id).await;
                    }
                    Err(e) => {
                        let err_msg = format!("failed to save file to lsys-file: {}", e);
                        error!("export_task: task id={} error: {}", task_id, err_msg);
                        Self::mark_failed(db, task_id, &err_msg).await;
                    }
                }
            }
        }
    }

    /// 标记任务为失败
    async fn mark_failed(db: &sqlx::Pool<sqlx::MySql>, task_id: u64, error_message: &str) {
        let now = now_time().unwrap_or(0);
        if let Err(e) = Update::<_, ExportTaskModel>::new()
            .set(ExportTaskModel::STATUS, ExportTaskStatus::Failed as i8)
            .set(ExportTaskModel::ERROR_MESSAGE, error_message)
            .set(ExportTaskModel::CHANGE_TIME, now)
            .execute(db, |qb| {
                qb.push_where().field_eq("id", task_id);
            })
            .await
        {
            error!(
                "export_task: failed to mark task id={} as Failed: {}",
                task_id, e
            );
        }
    }

    /// 标记任务为成功
    async fn mark_success(db: &sqlx::Pool<sqlx::MySql>, task_id: u64) {
        let now = now_time().unwrap_or(0);
        if let Err(e) = Update::<_, ExportTaskModel>::new()
            .set(ExportTaskModel::STATUS, ExportTaskStatus::Success as i8)
            .set(ExportTaskModel::ERROR_MESSAGE, "")
            .set(ExportTaskModel::CHANGE_TIME, now)
            .execute(db, |qb| {
                qb.push_where().field_eq("id", task_id);
            })
            .await
        {
            error!(
                "export_task: failed to mark task id={} as Success: {}",
                task_id, e
            );
        }
    }
}

impl ExportTaskDispatcher {
    /// 超时检测：将长时间处于 Running 的任务标记为 Failed
    ///
    /// - `timeout_secs`: 超过此秒数仍为 Running 的记录将被标记失败
    ///
    /// 返回受影响的行数。
    async fn mark_timeout_tasks(&self, timeout_secs: u64) -> FileManagerResult<u64> {
        let now = now_time().unwrap_or_default();
        let threshold = now.saturating_sub(timeout_secs);

        let affected = Update::<_, ExportTaskModel>::new()
            .set(ExportTaskModel::STATUS, ExportTaskStatus::Failed as i8)
            .set(
                ExportTaskModel::ERROR_MESSAGE,
                format!("timeout: exceeded {}s", timeout_secs),
            )
            .set(ExportTaskModel::CHANGE_TIME, now)
            .execute(&self.db, |qb| {
                qb.push_where()
                    .field_eq("status", ExportTaskStatus::Running as i8);
                qb.push_and().field_lt("add_time", threshold);
            })
            .await?
            .rows_affected();

        if affected > 0 {
            info!(
                "export_task: marked {} timed-out tasks as Failed (threshold={}s)",
                affected, timeout_secs
            );
        }

        Ok(affected)
    }
}
