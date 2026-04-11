// 导出任务查询 + 提交 + 超时检测

use lsys_core::db::{
    Insert, OffsetPageParam, QueryBuilderExt, TableMeta, Update, WhereClause, utils::FetchField,
};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, StringClear, now_time, string_clear};
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use lsys_file::dao::FileListAttrParam;
use lsys_file::model::FileModel;
use serde::Serialize;
use sqlx::{MySql, QueryBuilder};
use std::collections::HashMap;
use tracing::info;

use crate::dao::result::{FileManagerError, FileManagerResult};
use crate::model::*;

use super::ExportTask;
use super::logger::LogExportTask;

/// 提交导出任务的参数
#[derive(Clone)]
pub struct SubmitExportTaskParam<'a> {
    pub app_id: u64,
    pub app_user_id: u64,
    pub user_id: u64,
    pub add_user_id: u64,
    pub export_type: &'a str,
    pub params: &'a serde_json::Value,
    pub request_id: &'a str,
}

/// 导出任务关联的文件摘要（仅 Success 任务有关联文件）
///
/// 文件通过 TAG `export_{task_id}` 与任务关联，由 task.rs 在完成后写入。
#[derive(Debug, Clone, Serialize)]
pub struct ExportTaskFileItem {
    pub file_id: u64,
    pub file_name: String,
    pub file_size: u64,
    pub content_type: String,
    pub file_url: Option<String>,
}

/// 任务列表项 = 任务本身 + 可选文件信息（ATTR 方式附加，仅 Success 任务有文件）
#[derive(Debug, Clone, Serialize)]
pub struct ExportTaskItem {
    #[serde(flatten)]
    pub task: ExportTaskModel,
    pub file: Option<ExportTaskFileItem>,
}

/// 控制 list_tasks 是否附加额外数据（参考 FileListAttrParam 模式）
///
/// ```ignore
/// // 附加文件信息
/// ExportTaskListAttr { attr_file: Some(true) }
/// // 不附加文件信息（仅任务基本字段）
/// ExportTaskListAttr::default()
/// ```
#[derive(Debug, Default)]
pub struct ExportTaskListAttr {
    /// 为 `Some(true)` 时：对 Success 任务批量加载关联文件摘要（通过 TAG `export_{id}`）
    pub attr_file: Option<bool>,
}

impl ExportTask {
    /// 提交导出请求
    ///
    /// 1. 校验 export_type 是否已注册
    /// 2. 序列化 params 到 export_params
    /// 3. 插入 Pending 记录
    /// 4. 触发调度
    ///
    /// 注意：权限检查应在调用此方法之前在 Web 层完成
    pub async fn submit(
        &self,
        param: SubmitExportTaskParam<'_>,
        env_data: Option<&RequestEnv>,
    ) -> FileManagerResult<u64> {
        // 获取字段最大长度
        let export_type_max = FetchField::new(&self.db)
            .string_max::<ExportTaskModel>(&ExportTaskModel::EXPORT_TYPE)
            .await
            .len_or(64);
        let request_id_max = FetchField::new(&self.db)
            .string_max::<ExportTaskModel>(&ExportTaskModel::REQUEST_ID)
            .await
            .len_or(128);

        // 校验参数
        ValidParam::default()
            .add(
                valid_key!("export_type"),
                &param.export_type,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, export_type_max)),
            )
            .add(
                valid_key!("request_id"),
                &param.request_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, request_id_max)),
            )
            .check()?;

        // 校验 export_type 是否已注册
        self.exporters.get(param.export_type).ok_or_else(|| {
            FileManagerError::Message(fluent_message!(
                "export-type-not-registered",
                param.export_type
            ))
        })?;

        let export_params = serde_json::to_string(param.params).unwrap_or_default();
        let now = now_time().unwrap_or_default();

        let res = Insert::<_, ExportTaskModel>::new()
            .set(ExportTaskModel::APP_ID, param.app_id)
            .set(ExportTaskModel::APP_USER_ID, param.app_user_id)
            .set(ExportTaskModel::USER_ID, param.user_id)
            .set(ExportTaskModel::ADD_USER_ID, param.add_user_id)
            .set(ExportTaskModel::EXPORT_TYPE, param.export_type)
            .set(ExportTaskModel::EXPORT_PARAMS, &export_params)
            .set(ExportTaskModel::STATUS, ExportTaskStatus::Pending as i8)
            .set(ExportTaskModel::ERROR_MESSAGE, "")
            .set(ExportTaskModel::ADD_TIME, now)
            .set(ExportTaskModel::CHANGE_TIME, 0u64)
            .set(ExportTaskModel::REQUEST_ID, param.request_id)
            .execute(&self.db)
            .await?;

        let task_id = res.last_insert_id();

        info!(
            "export_task: submitted task id={}, type={}, app_id={}",
            task_id, param.export_type, param.app_id
        );

        self.logger
            .add(
                &LogExportTask {
                    action: "submit",
                    task_id,
                    app_id: param.app_id,
                    user_id: param.user_id,
                    add_user_id: param.add_user_id,
                    export_type: param.export_type,
                },
                Some(task_id),
                Some(param.add_user_id),
                None,
                env_data,
            )
            .await;

        // 触发调度
        self.trigger();

        Ok(task_id)
    }

    /// 按 ID 查询单条记录
    pub async fn find_by_id(&self, id: u64) -> FileManagerResult<Option<ExportTaskModel>> {
        let sql = format!("SELECT * FROM {} WHERE id=?", ExportTaskModel::table_name());
        let row = sqlx::query_as::<_, ExportTaskModel>(&sql)
            .bind(id)
            .fetch_optional(&self.db)
            .await?;
        Ok(row)
    }

    /// 用户维度任务列表（Offset 分页）
    ///
    /// - `user_id`: 用户 ID（必须，系统时为 0，用户端为当前登录用户 ID）
    /// - `app_id`: 应用 ID（可选，系统端为 Some(0)）
    /// - `export_type`: 可选类型过滤
    /// - `request_id`: 可选请求 ID 过滤
    /// - `status`: 可选状态过滤
    /// - `page`: Offset 分页参数
    ///
    /// Success 任务会通过 TAG `export_{id}` 批量加载关联文件（ATTR 方式附加）。
    #[allow(clippy::too_many_arguments)]
    pub async fn list_tasks(
        &self,
        user_id: u64,
        app_id: Option<u64>,
        export_type: Option<&str>,
        request_id: Option<&str>,
        status: Option<i8>,
        page: &OffsetPageParam,
        attr: &ExportTaskListAttr,
    ) -> FileManagerResult<Vec<ExportTaskItem>> {
        let mut qb =
            QueryBuilder::<MySql>::new(format!("SELECT * FROM {}", ExportTaskModel::table_name()));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_list_where(&mut wb, user_id, app_id, export_type, request_id, status);
        }
        qb.push(" ORDER BY id DESC");
        page.push_limit(&mut qb);

        let tasks: Vec<ExportTaskModel> = qb
            .build_query_as::<ExportTaskModel>()
            .fetch_all(&self.db)
            .await?;

        let mut file_map: HashMap<u64, ExportTaskFileItem> = HashMap::new();

        // ── ATTR：按需批量加载 Success 任务的关联文件 ────────────────────────────
        if attr.attr_file == Some(true) {
            let success_ids: Vec<u64> = tasks
                .iter()
                .filter(|t| t.status == ExportTaskStatus::Success as i8)
                .map(|t| t.id)
                .collect();

            if !success_ids.is_empty() {
                // 构建 tag_name 和 task_id 的映射
                let tag_names: Vec<String> = success_ids
                    .iter()
                    .map(|id| format!("export_id_{}", id))
                    .collect();
                let tag_refs: Vec<&str> = tag_names.iter().map(String::as_str).collect();

                // 建立 tag_name -> task_id 的映射，避免后续字符串解析
                let tag_to_id: std::collections::HashMap<String, u64> = success_ids
                    .iter()
                    .zip(tag_names.iter())
                    .map(|(id, tag)| (tag.clone(), *id))
                    .collect();

                let file_attr = FileListAttrParam {
                    attr_local: Some(true),
                    attr_oss: Some(false),
                    attr_tag: Some(false), // 不需要查询 tag 信息
                };

                // 使用批量查询，每个标签获取最多 1 个文件
                let batch_result = self
                    .file_dao
                    .data_dao()
                    .batch_list_files_by_tags(&tag_refs, Some(user_id), app_id, 1, &file_attr)
                    .await?;

                // 收集所有文件模型用于批量获取 URL
                let file_models: Vec<FileModel> = batch_result
                    .values()
                    .filter_map(|(files, _)| files.first())
                    .map(|item| FileModel {
                        id: item.item.file_id,
                        storage_type: item.item.storage_type.clone(),
                        status: item.item.status,
                        file_name: item.item.file_name.clone(),
                        file_md5: item.item.file_md5.clone(),
                        file_size: item.item.file_size,
                        content_type: item.item.content_type.clone(),
                        ..Default::default()
                    })
                    .collect();

                // 批量获取 URL
                let url_map = self
                    .file_dao
                    .get_file_urls(&file_models)
                    .await
                    .unwrap_or_else(|_| std::collections::HashMap::new());

                // 构建 tag_name → ExportTaskFileItem 的映射
                for (tag_name, (files, _has_more)) in batch_result {
                    if let Some(item) = files.first() {
                        let file_url = url_map.get(&item.item.file_id).cloned();

                        let file_item = ExportTaskFileItem {
                            file_id: item.item.file_id,
                            file_name: item.item.file_name.clone(),
                            file_size: item.item.file_size,
                            content_type: item.item.content_type.clone(),
                            file_url,
                        };

                        // 使用预先建立的映射，避免字符串解析
                        if let Some(&task_id) = tag_to_id.get(&tag_name) {
                            file_map.insert(task_id, file_item);
                        }
                    }
                }
            } // end if !success_ids.is_empty()
        } // end if attr.attr_file == Some(true)

        // ── 组装结果 ──────────────────────────────────────────────────────────
        let result = tasks
            .into_iter()
            .map(|task| {
                let file = file_map.remove(&task.id);
                ExportTaskItem { task, file }
            })
            .collect();

        Ok(result)
    }

    /// 用户维度任务总数
    pub async fn count_tasks(
        &self,
        user_id: u64,
        app_id: Option<u64>,
        export_type: Option<&str>,
        request_id: Option<&str>,
        status: Option<i8>,
    ) -> FileManagerResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT COUNT(*) FROM {}",
            ExportTaskModel::table_name()
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_list_where(&mut wb, user_id, app_id, export_type, request_id, status);
        }

        let count = qb
            .build_query_scalar()
            .fetch_one(&self.db)
            .await
            .unwrap_or(0i64);

        Ok(count)
    }

    /// 构建列表查询 WHERE 子句（以 user_id 为维度，app_id 可选）
    fn build_list_where<'a, 'args>(
        wb: &mut WhereClause<'a, 'args, MySql>,
        user_id: u64,
        app_id: Option<u64>,
        export_type: Option<&str>,
        request_id: Option<&str>,
        status: Option<i8>,
    ) {
        wb.and().field_eq("user_id", user_id);
        if let Some(aid) = app_id {
            wb.and().field_eq("app_id", aid);
        }
        if let Some(et) = export_type {
            let et = string_clear(et, StringClear::Ident, Some(512));
            if !et.is_empty() {
                wb.and().field_eq("export_type", et);
            }
        }
        if let Some(rid) = request_id {
            let rid = string_clear(rid, StringClear::Ident, Some(512));
            if !rid.is_empty() {
                wb.and().field_eq("request_id", rid);
            }
        }
        if let Some(s) = status {
            wb.and().field_eq("status", s);
        } else {
            // 排除已删除的记录
            wb.and().field_ne("status", ExportTaskStatus::Deleted as i8);
        }
    }

    /// 活跃任务数（Pending + Running）
    ///
    /// 用于前端轮询：初始化时调用一次，结果 > 0 时开始定时轮询，
    /// 返回 0 时停止轮询。
    ///
    /// - `user_id`: 用户 ID（系统时为 0）
    /// - `app_id`: 应用 ID（可选，系统时为 Some(0)）
    /// - `export_type`: 可选，仅统计指定类型的活跃任务
    pub async fn count_active_tasks(
        &self,
        user_id: u64,
        app_id: Option<u64>,
        export_type: Option<&str>,
    ) -> FileManagerResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT COUNT(*) FROM {}",
            ExportTaskModel::table_name()
        ));
        qb.push_where().field_eq("user_id", user_id);
        qb.push_and().field_in_copied(
            "status",
            &[
                ExportTaskStatus::Pending as i8,
                ExportTaskStatus::Running as i8,
            ],
        );

        if let Some(aid) = app_id {
            qb.push_and().field_eq("app_id", aid);
        }

        if let Some(et) = export_type {
            let et = string_clear(et, StringClear::Ident, Some(512));
            if !et.is_empty() {
                qb.push_and().field_eq("export_type", et);
            }
        }

        let count = qb
            .build_query_scalar()
            .fetch_one(&self.db)
            .await
            .unwrap_or(0i64);

        Ok(count)
    }

    /// 超时检测：将长时间处于 Running 的任务标记为 Failed
    ///
    /// - `timeout_secs`: 超过此秒数仍为 Running 的记录将被标记失败
    ///
    /// 返回受影响的行数。
    /// 建议由定时任务（如每分钟一次）调用。
    pub async fn mark_timeout_tasks(&self, timeout_secs: u64) -> FileManagerResult<u64> {
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

    /// 软删除任务（状态 → Deleted），以 user_id 为维度
    pub async fn delete_task(
        &self,
        task_id: u64,
        app_id: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> FileManagerResult<bool> {
        let now = now_time().unwrap_or_default();

        // 以 user_id 为维度删除，不能删除 Running 状态的
        let affected = Update::<_, ExportTaskModel>::new()
            .set(ExportTaskModel::STATUS, ExportTaskStatus::Deleted as i8)
            .set(ExportTaskModel::CHANGE_TIME, now)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", task_id);
                qb.push_and().field_eq("user_id", user_id);
                qb.push_and()
                    .field_ne("status", ExportTaskStatus::Running as i8);
            })
            .await?
            .rows_affected();

        if affected > 0 {
            self.logger
                .add(
                    &LogExportTask {
                        action: "delete",
                        task_id,
                        app_id,
                        user_id,
                        add_user_id: user_id,
                        export_type: "",
                    },
                    Some(task_id),
                    Some(user_id),
                    None,
                    env_data,
                )
                .await;
        }

        Ok(affected > 0)
    }
}
