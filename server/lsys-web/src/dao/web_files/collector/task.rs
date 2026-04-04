// 提交采集任务

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use lsys_core::db::{Insert, QueryBuilderExt, Update};
use lsys_core::fluent_message;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::{now_time, RequestEnv};
use lsys_files::dao::LocalFileMode;
use lsys_lib_jsrun::runner::{TaskOutcome, TaskResult};
use lsys_lib_jsrun::{
    FileLocalSyncHandler, LogHandler, MessageHandler, RuntimeConfig, MESSAGE_TYPE_GET_ENV,
    MESSAGE_TYPE_GET_PARAM,
};
use std::path::PathBuf;
use tracing::{error, info};

use crate::dao::result::{WebError, WebResult};
use crate::model::*;

use super::logger::LogCollectorTask;
use super::WebFileCollector;

impl WebFileCollector {
    /// 提交采集任务
    #[allow(clippy::too_many_arguments)]
    pub async fn submit_task(
        &self,
        script_id: u64,
        user_id: u64,
        add_user_id: u64,
        app_id: u64,
        request_id: &str,
        params: &serde_json::Value,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<(u64, u64, String)> {
        // 查询脚本
        let script = self
            .find_script_by_id(script_id)
            .await?
            .ok_or_else(|| WebError::Message(fluent_message!("collector-script-not-found")))?;

        if !CollectorScriptStatus::Enable.eq(script.status) {
            return Err(WebError::Message(fluent_message!(
                "collector-script-disabled"
            )));
        }

        let request_id = request_id.trim().to_string();
        if request_id.is_empty() {
            return Err(WebError::Message(fluent_message!(
                "collector-request-id-empty"
            )));
        }

        let now = now_time().unwrap_or_default();
        let params_json = serde_json::to_string(params).unwrap_or_default();

        // 插入执行记录（状态=Pending）
        let record_res = Insert::<_, CollectorRecordModel>::new()
            .set(CollectorRecordModel::REQUEST_ID, &request_id)
            .set(CollectorRecordModel::SCRIPT_ID, script_id)
            .set(CollectorRecordModel::ADD_USER_ID, add_user_id)
            .set(CollectorRecordModel::APP_ID, app_id)
            .set(CollectorRecordModel::TASK_ID, 0u64)
            .set(CollectorRecordModel::EXEC_PARAMS, &params_json)
            .set(
                CollectorRecordModel::STATUS,
                CollectorRecordStatus::Pending as i8,
            )
            .set(CollectorRecordModel::ELAPSED_MS, 0u64)
            .set(CollectorRecordModel::ERROR_MESSAGE, "")
            .set(CollectorRecordModel::ADD_TIME, now)
            .set(CollectorRecordModel::START_TIME, 0u64)
            .set(CollectorRecordModel::FINISH_TIME, 0u64)
            .execute(&self.db)
            .await?;

        let record_id = record_res.last_insert_id();

        // 构造 RuntimeConfig
        let work_dir = self
            .config
            .work_base_dir
            .join(format!("task_{}", record_id));
        let params_clone = params.clone();

        // message_handler: 透传 params 给 JS runtime.std.getParams，以及获取环境变量
        let message_handler: MessageHandler = Arc::new(move |_ns, msg_type, data| {
            let params = params_clone.clone();
            Box::pin(async move {
                if msg_type == MESSAGE_TYPE_GET_PARAM {
                    // 直接返回整个 params 对象
                    return params;
                }
                if msg_type == MESSAGE_TYPE_GET_ENV {
                    if let Some(key) = data.as_str()
                        && let Ok(val) = std::env::var(key) {
                            return serde_json::Value::String(val);
                        }
                    return serde_json::Value::Null;
                }
                serde_json::Value::Null
            })
        });

        // file_sync_handler: 调用 FileDao.create_from_local_file + 打 4 个 TAG
        let file_dao = self.file_dao.clone();
        let tag_script_name = format!("script_name_{}", script.name);
        let tag_script_md5 = format!("script_md5_{}", script.script_md5);
        let tag_script_id = format!("script_id_{}", script.id);
        let tag_request_id = format!("request_{}", request_id);
       
        let file_sync_handler: FileLocalSyncHandler = Arc::new(
            move |_ns: Option<String>, file_path: PathBuf, _work_dir: PathBuf| {
                let file_dao = file_dao.clone();
                let tag1 = tag_script_name.clone();
                let tag2 = tag_script_md5.clone();
                let tag3 = tag_script_id.clone();
                let tag4 = tag_request_id.clone();

                Box::pin(async move {
                    let path_str = file_path.to_string_lossy().to_string();
                    let tag_names: Vec<&str> = vec![&tag1, &tag2, &tag3, &tag4];

                    let (file_model, _file_user) = file_dao
                        .create_from_local_file(
                            &path_str,
                            user_id,
                            add_user_id,
                            app_id,
                            None,
                            LocalFileMode::Move,
                            None,
                            &tag_names,
                            None,
                        )
                        .await
                        .map_err(|e| format!("file sync error: {}", e))?;

                    Ok(serde_json::json!({
                        "file_id": file_model.id,
                        "file_md5": file_model.file_md5,
                        "file_name": file_model.file_name,
                        "file_size": file_model.file_size,
                    }))
                })
                    as Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send>>
            },
        );

        // log_handler: 将 JS 脚本日志写入 lst_collector_log
        let db_for_log = self.db.clone();
        let log_request_id = request_id.clone();
        let log_script_id = script_id;
        let log_user_id = add_user_id;
        let log_app_id = app_id;

        let log_handler: LogHandler = Arc::new(
            move |_ns: Option<String>, log_level: u32, message: String| {
                let db = db_for_log.clone();
                let rid = log_request_id.clone();
                Box::pin(async move {
                    let level = log_level as u8;
                    WebFileCollector::add_log_raw(
                        &db,
                        &rid,
                        log_script_id,
                        log_user_id,
                        log_app_id,
                        level,
                        &message,
                    )
                    .await;
                })
            },
        );

        let runtime_config = RuntimeConfig {
            memory_limit: script.memory_limit as usize,
            execution_timeout: std::time::Duration::from_secs(script.timeout_secs as u64),
            work_dir,
            message_handler: Some(message_handler),
            file_sync_handler: Some(file_sync_handler),
            log_handler: Some(log_handler),
            namespace: Some(format!("collector_{}", script.id)),
            ..RuntimeConfig::default()
        };

        // 在提交任务前先将记录状态置为 Running，避免任务完成后 callback 的 UPDATE
        // 被后续的 "更新为 Running" 覆盖（竞态条件）。
        let start_now = now_time().unwrap_or_default();

        if let Err(e) = Update::<_, CollectorRecordModel>::new()
            .set(
                CollectorRecordModel::STATUS,
                CollectorRecordStatus::Running as i8,
            )
            .set(CollectorRecordModel::START_TIME, start_now)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", record_id);
            })
            .await
        {
            error!(
                "collector submit: failed to update record id={} to Running: {}",
                record_id, e
            );
        }

        // 提交任务到 JsTaskRunner，附带 callback 更新记录
        let db = self.db.clone();
        let cb_record_id = record_id;
        let cb_request_id = request_id.clone();
        let cb_script_id = script_id;
        let cb_user_id = add_user_id;
        let cb_app_id = app_id;

        let handle = self.runner.submit(
            &script.script_code,
            Some(runtime_config),
            Some(move |result: TaskResult| {
                let db = db.clone();
                let task_id = result.task_id;
                let elapsed_ms = result.elapsed.as_millis() as u64;
                let outcome = result.outcome.clone();

                async move {
                    let finish_now = now_time().unwrap_or_default();
                    let (status, error_msg) = match &outcome {
                        TaskOutcome::Success(_) => {
                            (CollectorRecordStatus::Success as i8, String::new())
                        }
                        TaskOutcome::Error(e) => {
                            if e.contains("timed out") {
                                (CollectorRecordStatus::Timeout as i8, e.clone())
                            } else {
                                (CollectorRecordStatus::Failed as i8, e.clone())
                            }
                        }
                    };

                    let update_result = Update::<_, CollectorRecordModel>::new()
                        .set(CollectorRecordModel::TASK_ID, task_id)
                        .set(CollectorRecordModel::STATUS, status)
                        .set(CollectorRecordModel::ELAPSED_MS, elapsed_ms)
                        .set(CollectorRecordModel::ERROR_MESSAGE, &error_msg)
                        .set(CollectorRecordModel::FINISH_TIME, finish_now)
                        .execute(&db, |qb| {
                            qb.push_where().field_eq("id", cb_record_id);
                        })
                        .await;

                    if let Err(e) = update_result {
                        error!(
                            "collector callback: failed to update record id={}, request_id={}: {}",
                            cb_record_id, cb_request_id, e
                        );
                    } else {
                        info!(
                            "collector callback: record id={} request_id={} status={} elapsed={}ms",
                            cb_record_id, cb_request_id, status, elapsed_ms
                        );
                    }

                    // 写入系统日志：任务完成
                    let sys_msg = match &outcome {
                        TaskOutcome::Success(_) => {
                            format!("task completed: success, elapsed={}ms", elapsed_ms)
                        }
                        TaskOutcome::Error(e) => {
                            format!("task completed: error={}, elapsed={}ms", e, elapsed_ms)
                        }
                    };
                    WebFileCollector::add_log_raw(
                        &db,
                        &cb_request_id,
                        cb_script_id,
                        cb_user_id,
                        cb_app_id,
                        COLLECTOR_LOG_LEVEL_SYSTEM,
                        &sys_msg,
                    )
                    .await;
                }
            }),
        ).await;

        let task_id = handle.task_id;

        // 回填 task_id（callback 也会写入相同值，此处仅作补充，不涉及状态）
        if let Err(e) = Update::<_, CollectorRecordModel>::new()
            .set(CollectorRecordModel::TASK_ID, task_id)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", record_id);
            })
            .await
        {
            error!(
                "collector submit: failed to update task_id for record id={}: {}",
                record_id, e
            );
            WebFileCollector::add_log_raw(
                &self.db,
                &request_id,
                script_id,
                add_user_id,
                app_id,
                COLLECTOR_LOG_LEVEL_SYSTEM,
                &format!(
                    "failed to update task_id for record id={}: {}",
                    record_id, e
                ),
            )
            .await;
        }

        // 写入系统日志：任务已提交
        if let Err(e) = self
            .add_log(
                &request_id,
                script_id,
                add_user_id,
                app_id,
                COLLECTOR_LOG_LEVEL_SYSTEM,
                &format!(
                    "task submitted: record_id={}, task_id={}, script={}",
                    record_id, task_id, script.name
                ),
            )
            .await
        {
            error!(
                "collector submit: failed to write submit log for record id={}: {}",
                record_id,
                e.to_fluent_message().default_format()
            );
        }

        self.logger
            .add(
                &LogCollectorTask {
                    action: "submit",
                    record_id,
                    task_id,
                    script_id,
                    script_name: &script.name,
                    user_id: add_user_id,
                    app_id,
                    request_id: &request_id,
                },
                Some(record_id),
                Some(add_user_id),
                None,
                env_data,
            )
            .await;

        Ok((record_id, task_id, script.name.clone()))
    }
}
