// 脚本 CRUD + 列表 + 计数 + 脚本文件查询

use lsys_core::db::{
    CursorPageData, CursorPageParam, Insert, OffsetPageParam, SqlQuote, SqlSuffix, TableMeta,
    Update,
};
use lsys_core::fluent_message;
use lsys_core::sql_format;
use lsys_core::utils::{now_time, RequestEnv};
use lsys_files::dao::{FileDataListParam, FileListAttrParam};

use crate::dao::result::{WebError, WebResult};
use crate::model::*;

use super::logger::LogCollectorScript;
use super::WebFileCollector;

/// 脚本关联文件的标签信息
#[derive(Debug, Clone)]
pub struct ScriptFileTag {
    pub tag_name: String,
    pub add_time: u64,
}

/// 脚本关联的文件信息（含标签 + URL）
#[derive(Debug, Clone)]
pub struct ScriptFileItem {
    pub file_id: u64,
    pub file_name: String,
    pub file_md5: String,
    pub file_size: u64,
    pub storage_type: String,
    pub content_type: String,
    pub file_url: Option<String>,
    pub add_time: u64,
    pub tags: Vec<ScriptFileTag>,
}

impl WebFileCollector {
    /// 创建脚本
    #[allow(clippy::too_many_arguments)]
    pub async fn script_add(
        &self,
        user_id: u64,
        app_id: u64,
        name: &str,
        script_code: &str,
        timeout_secs: Option<u32>,
        memory_limit: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<u64> {
        let name = name.trim();
        if name.is_empty() {
            return Err(WebError::Message(fluent_message!(
                "collector-script-name-empty"
            )));
        }
        if script_code.trim().is_empty() {
            return Err(WebError::Message(fluent_message!(
                "collector-script-code-empty"
            )));
        }
        if let Err(err) = lsys_lib_jsrun::check_js_syntax(script_code) {
            return Err(WebError::Message(fluent_message!(
                "collector-script-syntax-error",
                err
            )));
        }

        let now = now_time().map_err(|e| WebError::Message(fluent_message!("time-error", e)))?;
        let script_md5 = format!("{:x}", md5::compute(script_code.as_bytes()));

        let res = Insert::<_, CollectorScriptModel>::new()
            .set(CollectorScriptModel::USER_ID, user_id)
            .set(CollectorScriptModel::APP_ID, app_id)
            .set(CollectorScriptModel::NAME, name)
            .set(CollectorScriptModel::SCRIPT_CODE, script_code)
            .set(CollectorScriptModel::SCRIPT_MD5, &script_md5)
            .set(
                CollectorScriptModel::TIMEOUT_SECS,
                timeout_secs.unwrap_or(30),
            )
            .set(
                CollectorScriptModel::MEMORY_LIMIT,
                memory_limit.unwrap_or(64 * 1024 * 1024),
            )
            .set(
                CollectorScriptModel::STATUS,
                CollectorScriptStatus::Enable as i8,
            )
            .set(CollectorScriptModel::ADD_TIME, now)
            .set(CollectorScriptModel::CHANGE_TIME, 0u64)
            .execute(&self.db)
            .await?;

        let script_id = res.last_insert_id();

        self.logger
            .add(
                &LogCollectorScript {
                    action: "add",
                    script_id,
                    user_id,
                    app_id,
                    name,
                },
                Some(script_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(script_id)
    }

    /// 更新脚本
    pub async fn script_edit(
        &self,
        script_id: u64,
        name: &str,
        script_code: &str,
        timeout_secs: Option<u32>,
        memory_limit: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<u64> {
        let name = name.trim();
        if name.is_empty() {
            return Err(WebError::Message(fluent_message!(
                "collector-script-name-empty"
            )));
        }
        if script_code.trim().is_empty() {
            return Err(WebError::Message(fluent_message!(
                "collector-script-code-empty"
            )));
        }
        if let Err(err) = lsys_lib_jsrun::check_js_syntax(script_code) {
            return Err(WebError::Message(fluent_message!(
                "collector-script-syntax-error",
                err
            )));
        }

        let now = now_time().map_err(|e| WebError::Message(fluent_message!("time-error", e)))?;
        let script_md5 = format!("{:x}", md5::compute(script_code.as_bytes()));

        let where_sql = sql_format!(
            "id={} AND status!={}",
            script_id,
            CollectorScriptStatus::Deleted as i8
        );

        let res = Update::<_, CollectorScriptModel>::new()
            .set(CollectorScriptModel::NAME, name)
            .set(CollectorScriptModel::SCRIPT_CODE, script_code)
            .set(CollectorScriptModel::SCRIPT_MD5, &script_md5)
            .set(
                CollectorScriptModel::TIMEOUT_SECS,
                timeout_secs.unwrap_or(30),
            )
            .set(
                CollectorScriptModel::MEMORY_LIMIT,
                memory_limit.unwrap_or(64 * 1024 * 1024),
            )
            .set(CollectorScriptModel::CHANGE_TIME, now)
            .execute(SqlSuffix::Where(&where_sql), &self.db)
            .await?;

        self.logger
            .add(
                &LogCollectorScript {
                    action: "edit",
                    script_id,
                    user_id: 0,
                    app_id: 0,
                    name,
                },
                Some(script_id),
                None,
                None,
                env_data,
            )
            .await;

        Ok(res.rows_affected())
    }

    /// 修改脚本状态（启用/禁用）
    pub async fn script_change_status(
        &self,
        script_id: u64,
        status: CollectorScriptStatus,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<u64> {
        let now = now_time().map_err(|e| WebError::Message(fluent_message!("time-error", e)))?;

        let where_sql = sql_format!(
            "id={} AND status!={}",
            script_id,
            CollectorScriptStatus::Deleted as i8
        );

        let res = Update::<_, CollectorScriptModel>::new()
            .set(CollectorScriptModel::STATUS, status as i8)
            .set(CollectorScriptModel::CHANGE_TIME, now)
            .execute(SqlSuffix::Where(&where_sql), &self.db)
            .await?;

        let action = match status {
            CollectorScriptStatus::Enable => "enable",
            CollectorScriptStatus::Disable => "disable",
            CollectorScriptStatus::Deleted => "delete",
        };
        self.logger
            .add(
                &LogCollectorScript {
                    action,
                    script_id,
                    user_id: 0,
                    app_id: 0,
                    name: "",
                },
                Some(script_id),
                None,
                None,
                env_data,
            )
            .await;

        Ok(res.rows_affected())
    }

    /// 删除脚本（软删除）
    pub async fn script_delete(
        &self,
        script_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<u64> {
        self.script_change_status(script_id, CollectorScriptStatus::Deleted, env_data)
            .await
    }

    /// 按 ID 查询脚本
    pub async fn find_script_by_id(
        &self,
        script_id: u64,
    ) -> WebResult<Option<CollectorScriptModel>> {
        let sql = sql_format!(
            "SELECT * FROM {} WHERE id={}",
            CollectorScriptModel::table_name(),
            script_id
        );
        let row = sqlx::query_as::<_, CollectorScriptModel>(&sql)
            .fetch_optional(&self.db)
            .await?;
        Ok(row)
    }

    /// 构建脚本查询的 WHERE 子句
    fn build_script_where(app_id: u64, status: Option<i8>) -> String {
        let mut clauses: Vec<String> = vec![
            sql_format!("app_id={}", app_id),
            sql_format!("status!={}", CollectorScriptStatus::Deleted as i8),
        ];
        if let Some(s) = status {
            clauses.push(sql_format!("status={}", s));
        }
        clauses.join(" AND ")
    }

    /// 脚本列表（Offset 分页）
    pub async fn list_scripts(
        &self,
        app_id: u64,
        status: Option<i8>,
        page: &OffsetPageParam,
    ) -> WebResult<Vec<CollectorScriptModel>> {
        let where_str = Self::build_script_where(app_id, status);
        let limit_sql = page.page_query().limit_sql().unwrap_or_default();

        let sql = format!(
            "SELECT * FROM {} WHERE {} ORDER BY id DESC{}",
            CollectorScriptModel::table_name().sql_quote(),
            where_str,
            limit_sql
        );

        let data = sqlx::query_as::<_, CollectorScriptModel>(&sql)
            .fetch_all(&self.db)
            .await?;

        Ok(data)
    }

    /// 脚本总数
    pub async fn count_scripts(&self, app_id: u64, status: Option<i8>) -> WebResult<u64> {
        let where_str = Self::build_script_where(app_id, status);

        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE {}",
            CollectorScriptModel::table_name().sql_quote(),
            where_str
        );

        let count = sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?;
        Ok(count as u64)
    }

    /// 某脚本下所有生成文件列表（通过 tag "script_id_{id}" 关联，CursorPage 分页）
    ///
    /// `app_id` 用于文件查询的 app_id 过滤（用户级接口传 Some，系统级可传 None）
    pub async fn list_script_files(
        &self,
        script: &CollectorScriptModel,
        page: &CursorPageParam<u64>,
        app_id: Option<u64>,
    ) -> WebResult<(Vec<ScriptFileItem>, CursorPageData<u64>)> {
        let tag_name = format!("script_id_{}", script.id);
        let tag_refs: Vec<&str> = vec![&tag_name];

        let file_filter = FileDataListParam {
            app_id,
            tag_names: Some(&tag_refs),
            ..Default::default()
        };
        let file_attr = FileListAttrParam {
            attr_local: Some(true),
            attr_oss: Some(false),
            attr_tag: Some(true),
        };

        let (files, page_data) = self
            .file_dao
            .data_dao()
            .list_files(&file_filter, page, &file_attr)
            .await?;

        // 批量获取 URL
        let file_models: Vec<lsys_files::model::FileModel> = files
            .iter()
            .map(|item| lsys_files::model::FileModel {
                id: item.item.id,
                storage_type: item.item.storage_type.clone(),
                status: item.item.status,
                file_name: item.item.file_name.clone(),
                file_md5: item.item.file_md5.clone(),
                file_size: item.item.file_size,
                modify_time: item.item.modify_time,
                content_type: item.item.content_type.clone(),
                copy_file_id: item.item.copy_file_id,
                from_user_id: item.item.from_user_id,
                add_time: item.item.add_time,
                change_time: item.item.change_time,
            })
            .collect();
        let url_map = self
            .file_dao
            .get_file_urls(&file_models)
            .await
            .unwrap_or_default();

        let items: Vec<ScriptFileItem> = files
            .iter()
            .map(|item| {
                let file_url = url_map.get(&item.item.id).cloned();
                let tags: Vec<ScriptFileTag> = item
                    .attr_tag
                    .as_ref()
                    .map(|t| {
                        t.tags
                            .iter()
                            .map(|tag| ScriptFileTag {
                                tag_name: tag.tag_name.clone(),
                                add_time: tag.add_time,
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                ScriptFileItem {
                    file_id: item.item.id,
                    file_name: item.item.file_name.clone(),
                    file_md5: item.item.file_md5.clone(),
                    file_size: item.item.file_size,
                    storage_type: item.item.storage_type.clone(),
                    content_type: item.item.content_type.clone(),
                    file_url,
                    add_time: item.item.file_user_add_time,
                    tags,
                }
            })
            .collect();

        Ok((items, page_data))
    }

    /// 某脚本下的生成文件总数
    pub async fn count_script_files(
        &self,
        script: &CollectorScriptModel,
        app_id: Option<u64>,
    ) -> WebResult<u64> {
        let tag_name = format!("script_id_{}", script.id);
        let tag_refs: Vec<&str> = vec![&tag_name];

        let file_filter = FileDataListParam {
            app_id,
            tag_names: Some(&tag_refs),
            ..Default::default()
        };

        let count = self.file_dao.data_dao().count_files(&file_filter).await?;

        Ok(count as u64)
    }
}
