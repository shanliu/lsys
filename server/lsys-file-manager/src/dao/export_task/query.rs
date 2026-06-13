// 导出任务查询 + 提交 + 超时检测

use lsys_core::db::{
    Insert, OffsetPageParam, QueryBuilderExt, TableMeta, Update, WhereClause, utils::FetchField,
};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, StringClear, now_time, string_clear};
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use serde::Serialize;
use sqlx::{MySql, QueryBuilder};
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
    pub lang: &'a str,
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

/// 任务列表项 = 任务本身 + 可选文件信息
#[derive(Debug, Clone, Serialize)]
pub struct ExportTaskItem {
    pub task: ExportTaskModel,
    pub file: Option<ExportTaskFileItem>,
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
            .set(ExportTaskModel::LANG, param.lang)
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
        user_id: Option<u64>,
        app_id: Option<u64>,
        export_type: Option<&str>,
        request_id: Option<&str>,
        status: Option<i8>,
        page: &OffsetPageParam,
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

        // 不再加载文件信息，使用 read_export_file 专门读取私有文件
        let result = tasks
            .into_iter()
            .map(|task| ExportTaskItem { task, file: None })
            .collect();

        Ok(result)
    }

    /// 用户维度任务总数
    pub async fn count_tasks(
        &self,
        user_id: Option<u64>,
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
        user_id: Option<u64>,
        app_id: Option<u64>,
        export_type: Option<&str>,
        request_id: Option<&str>,
        status: Option<i8>,
    ) {
        if let Some(uid) = user_id {
            wb.and().field_eq("user_id", uid);
        }
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
    /// - `user_id`: 用户 ID（可选，None 表示不按用户过滤）
    /// - `app_id`: 应用 ID（可选，系统时为 Some(0)）
    /// - `export_type`: 可选，仅统计指定类型的活跃任务
    pub async fn count_active_tasks(
        &self,
        user_id: Option<u64>,
        app_id: Option<u64>,
        export_type: Option<&str>,
    ) -> FileManagerResult<i64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT COUNT(*) FROM {}",
            ExportTaskModel::table_name()
        ));
        
        qb.push_where().field_in_copied(
            "status",
            &[
                ExportTaskStatus::Pending as i8,
                ExportTaskStatus::Running as i8,
            ],
        );

        if let Some(uid) = user_id {
            qb.push_and().field_eq("user_id", uid);
        }

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

    /// 读取导出任务的文件（支持偏移）
    ///
    /// 用于下载导出任务生成的文件，支持断点续传。
    /// 注意：此函数不做权限校验，权限校验应在调用层完成。
    ///
    /// # 参数
    /// - `task`: 任务模型（调用方已查询并校验权限）
    /// - `offset`: 读取起始偏移（字节）
    ///
    /// # 返回
    /// - `Ok((FileReadIterator, FileModel))`: 文件读取迭代器及文件模型
    /// - `Err`: 任务未完成、或文件不存在
    ///
    /// # 示例
    /// ```rust,ignore
    /// let task = export_task.find_by_id(task_id).await?;
    /// // 进行权限校验...
    /// let (mut iter, file_model) =
    ///     export_task.read_export_file(&task, 0).await?;
    /// while let Some(result) = iter.next_chunk().await {
    ///     let chunk = result?;
    ///     // 处理 chunk.data
    /// }
    /// ```
    pub async fn read_export_file(
        &self,
        task: &ExportTaskModel,
        offset: u64,
    ) -> FileManagerResult<(
        lsys_file::dao::FileReadIterator,
        lsys_file::model::FileModel,
    )> {
        // 任务必须是 Success 状态
        if task.status != ExportTaskStatus::Success as i8 {
            return Err(FileManagerError::Message(fluent_message!(
                "export-task-not-completed"
            )));
        }

        // 查询关联的文件（通过 TAG `export_id_{task_id}`）
        let tag_name = format!("export_id_{}", task.id);
        let file_attr = lsys_file::dao::FileListAttrParam {
            attr_local: Some(true),
            ..Default::default()
        };

        // 使用 CursorPageParam 获取第一个文件
        let page = lsys_core::db::CursorPageParam::new(
            lsys_core::db::CursorPageDir::Next,
            lsys_core::db::CursorConfig::primary(lsys_core::db::CursorPageSort::Desc),
            None,
            lsys_core::db::CursorLimit::Limit {
                limit: 1,
                more: false,
            },
        );
        let (files, _) = self
            .file_dao
            .data_dao()
            .list_files_by_tag(
                &tag_name,
                Some(task.user_id),
                Some(task.app_id),
                &page,
                &file_attr,
            )
            .await?;

        let file_item = files
            .first()
            .ok_or_else(|| FileManagerError::Message(fluent_message!("export-file-not-found")))?;

        // 构建 FileModel
        let file_model = lsys_file::model::FileModel {
            id: file_item.item.file_id,
            storage_type: file_item.item.storage_type.clone(),
            status: file_item.item.status,
            origin_name: file_item.item.file_name.clone(),
            file_md5: file_item.item.file_md5.clone(),
            file_size: file_item.item.file_size,
            content_type: file_item.item.content_type.clone(),
            ..Default::default()
        };

        // 使用 read_local_file 读取文件
        let iter = self
            .file_dao
            .data_dao()
            .read_local_file(&file_model, offset, None)
            .await?;

        Ok((iter, file_model))
    }
}
