// 脚本 CRUD + 列表 + 计数 + 脚本文件查询

use lsys_core::db::{
    CursorPageData, CursorPageParam, Insert, OffsetPageParam, QueryBuilderExt, TableMeta,
    TotalParam, TotalRow, Update, WhereClause,
    utils::FetchField,
};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, now_time};
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidStrlen};
use lsys_core::valid_key;
use lsys_file::dao::FileListAttrParam;
use sqlx::{MySql, QueryBuilder};

use crate::dao::result::{FileManagerError, FileManagerResult};
use crate::model::*;

use super::FileCollector;
use super::logger::LogCollectorScript;

/// 脚本关联文件的标签信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScriptFileTag {
    pub tag_name: String,
    pub add_time: u64,
}

/// 脚本关联的文件信息（含标签 + URL）
#[derive(Debug, Clone, serde::Serialize)]
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
    // 新增字段
    pub user_id: u64,
    pub add_user_id: u64,
    pub app_id: u64,
    pub status: i8,
    pub file_user_status: i8,
    pub source_url: String,
    pub source_md5: String,
    pub modify_time: u64,
    pub attr_local: Option<lsys_file::dao::FileLocalAttr>,
    pub attr_oss: Option<lsys_file::dao::FileOssAttr>,
}

impl FileCollector {
    /// 创建脚本
    #[allow(clippy::too_many_arguments)]
    pub async fn script_add(
        &self,
        add_user_id: u64,
        app_id: u64,
        app_user_id: u64,
        name: &str,
        script_code: &str,
        timeout_secs: Option<u32>,
        memory_limit: Option<u64>,
        env_data: Option<&RequestEnv>,
    ) -> FileManagerResult<u64> {
        // 从表结构获取字段最大长度
        let fetch_field = FetchField::new(&self.db);
        let name_max = fetch_field
            .string_max::<CollectorScriptModel>(&CollectorScriptModel::NAME)
            .await
            .len_or(100);
        let script_code_max = fetch_field
            .string_max::<CollectorScriptModel>(&CollectorScriptModel::SCRIPT_CODE)
            .await
            .len_or(16_777_215); // MEDIUMTEXT 默认最大长度
        let script_md5_max = fetch_field
            .string_max::<CollectorScriptModel>(&CollectorScriptModel::SCRIPT_MD5)
            .await
            .len_or(32);

        // 使用统一的验证方式
        ValidParam::default()
            .add(
                valid_key!("name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, name_max)),
            )
            .add(
                valid_key!("script_code"),
                &script_code,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, script_code_max)),
            )
            .check()?;

        if let Err(err) = lsys_lib_jsrun::check_js_syntax(script_code) {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-syntax-error",
                err
            )));
        }

        let timeout_val = timeout_secs.unwrap_or(30);
        let memory_val = memory_limit.unwrap_or(64 * 1024 * 1024);

        if self.config.max_timeout_secs > 0 && timeout_val > self.config.max_timeout_secs {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-timeout-exceed",
                {
                    "max": self.config.max_timeout_secs,
                    "val": timeout_val
                }
            )));
        }
        if self.config.max_memory_limit > 0 && memory_val > self.config.max_memory_limit {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-memory-exceed",
                {
                    "max": self.config.max_memory_limit,
                    "val": memory_val
                }
            )));
        }

        let now = now_time().unwrap_or_default();
        let script_md5 = format!("{:x}", md5::compute(script_code.as_bytes()));

        // 验证 MD5 长度
        if script_md5.len() > script_md5_max as usize {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-md5-too-long"
            )));
        }

        let res = Insert::<_, CollectorScriptModel>::new()
            .set(CollectorScriptModel::ADD_USER_ID, add_user_id)
            .set(CollectorScriptModel::APP_ID, app_id)
            .set(CollectorScriptModel::APP_USER_ID, app_user_id)
            .set(CollectorScriptModel::NAME, name)
            .set(CollectorScriptModel::SCRIPT_CODE, script_code)
            .set(CollectorScriptModel::SCRIPT_MD5, &script_md5)
            .set(CollectorScriptModel::TIMEOUT_SECS, timeout_val)
            .set(CollectorScriptModel::MEMORY_LIMIT, memory_val)
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
                    user_id: add_user_id,
                    app_id,
                    name,
                },
                Some(script_id),
                Some(add_user_id),
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
    ) -> FileManagerResult<u64> {
        // 从表结构获取字段最大长度
        let fetch_field = FetchField::new(&self.db);
        let name_max = fetch_field
            .string_max::<CollectorScriptModel>(&CollectorScriptModel::NAME)
            .await
            .len_or(100);
        let script_code_max = fetch_field
            .string_max::<CollectorScriptModel>(&CollectorScriptModel::SCRIPT_CODE)
            .await
            .len_or(16_777_215); // MEDIUMTEXT 默认最大长度
        let script_md5_max = fetch_field
            .string_max::<CollectorScriptModel>(&CollectorScriptModel::SCRIPT_MD5)
            .await
            .len_or(32);

        // 使用统一的验证方式
        ValidParam::default()
            .add(
                valid_key!("name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, name_max)),
            )
            .add(
                valid_key!("script_code"),
                &script_code,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, script_code_max)),
            )
            .check()?;

        if let Err(err) = lsys_lib_jsrun::check_js_syntax(script_code) {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-syntax-error",
                err
            )));
        }

        let timeout_val = timeout_secs.unwrap_or(30);
        let memory_val = memory_limit.unwrap_or(64 * 1024 * 1024);

        if self.config.max_timeout_secs > 0 && timeout_val > self.config.max_timeout_secs {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-timeout-exceed",
                {
                    "max": self.config.max_timeout_secs,
                    "val": timeout_val
                }
            )));
        }
        if self.config.max_memory_limit > 0 && memory_val > self.config.max_memory_limit {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-memory-exceed",
                {
                    "max": self.config.max_memory_limit,
                    "val": memory_val
                }
            )));
        }

        let now = now_time().unwrap_or_default();
        let script_md5 = format!("{:x}", md5::compute(script_code.as_bytes()));

        // 验证 MD5 长度
        if script_md5.len() > script_md5_max as usize {
            return Err(FileManagerError::Message(fluent_message!(
                "collector-script-md5-too-long"
            )));
        }

        let res = Update::<_, CollectorScriptModel>::new()
            .set(CollectorScriptModel::NAME, name)
            .set(CollectorScriptModel::SCRIPT_CODE, script_code)
            .set(CollectorScriptModel::SCRIPT_MD5, &script_md5)
            .set(CollectorScriptModel::TIMEOUT_SECS, timeout_val)
            .set(CollectorScriptModel::MEMORY_LIMIT, memory_val)
            .set(CollectorScriptModel::CHANGE_TIME, now)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", script_id);
                qb.push_and()
                    .field_ne("status", CollectorScriptStatus::Deleted as i8);
            })
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
    ) -> FileManagerResult<u64> {
        let now = now_time().unwrap_or_default();

        let res = Update::<_, CollectorScriptModel>::new()
            .set(CollectorScriptModel::STATUS, status as i8)
            .set(CollectorScriptModel::CHANGE_TIME, now)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", script_id);
                qb.push_and()
                    .field_ne("status", CollectorScriptStatus::Deleted as i8);
            })
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
    ) -> FileManagerResult<u64> {
        self.script_change_status(script_id, CollectorScriptStatus::Deleted, env_data)
            .await
    }

    /// 按 ID 查询脚本
    pub async fn find_script_by_id(
        &self,
        script_id: u64,
    ) -> FileManagerResult<Option<CollectorScriptModel>> {
        let sql = format!(
            "SELECT * FROM {} WHERE id=?",
            CollectorScriptModel::table_name()
        );
        let row = sqlx::query_as::<_, CollectorScriptModel>(&sql)
            .bind(script_id)
            .fetch_optional(&self.db)
            .await?;
        Ok(row)
    }

    /// 构建脚本查询的 WHERE 子句
    fn build_script_where<'a, 'args>(
        wb: &mut WhereClause<'a, 'args, MySql>,
        app_id: u64,
        status: Option<i8>,
    ) {
        wb.and().field_eq("app_id", app_id);
        wb.and()
            .field_ne("status", CollectorScriptStatus::Deleted as i8);
        if let Some(s) = status {
            wb.and().field_eq("status", s);
        }
    }

    /// 脚本列表（Offset 分页）
    pub async fn list_scripts(
        &self,
        app_id: u64,
        status: Option<i8>,
        page: &OffsetPageParam,
    ) -> FileManagerResult<Vec<CollectorScriptModel>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT * FROM {}",
            CollectorScriptModel::table_name()
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_script_where(&mut wb, app_id, status);
        }
        qb.push(" ORDER BY id DESC");
        page.push_limit(&mut qb);

        let data = qb
            .build_query_as::<CollectorScriptModel>()
            .fetch_all(&self.db)
            .await?;

        Ok(data)
    }

    /// 脚本总数
    pub async fn count_scripts(&self, app_id: u64, status: Option<i8>) -> FileManagerResult<u64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT COUNT(*) FROM {}",
            CollectorScriptModel::table_name()
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_script_where(&mut wb, app_id, status);
        }

        let count = qb
            .build_query_scalar()
            .fetch_one(&self.db)
            .await
            .unwrap_or(0i64);
        Ok(count as u64)
    }

    /// 某脚本下所有生成文件列表（通过 tag "script_id_{id}" 关联，CursorPage 分页）
    ///
    /// `app_id` 用于文件查询的 app_id 过滤（用户级接口传 Some，系统级可传 None）
    /// `attr_param` 用于控制是否查询文件的 local/oss/tag 属性
    pub async fn list_script_files(
        &self,
        script: &CollectorScriptModel,
        page: &CursorPageParam<u64>,
        app_id: Option<u64>,
        attr_param: &FileListAttrParam,
    ) -> FileManagerResult<(Vec<ScriptFileItem>, CursorPageData<u64>)> {
        let tag_name = format!("script_id_{}", script.id);

        let (files, page_data): (Vec<lsys_file::dao::FileListItemAttr>, _) = self
            .file_dao
            .data_dao()
            .list_files_by_tag(&tag_name, None, app_id, page, attr_param)
            .await?;

        // 批量获取 URL
        let file_models: Vec<lsys_file::model::FileModel> = files
            .iter()
            .map(|item| lsys_file::model::FileModel {
                id: item.item.file_id,
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
            .unwrap_or_else(|_| std::collections::HashMap::new());

        let items: Vec<ScriptFileItem> = files
            .iter()
            .map(|item| {
                let file_url = url_map.get(&item.item.file_id).cloned();
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
                    file_id: item.item.file_id,
                    file_name: item.item.file_name.clone(),
                    file_md5: item.item.file_md5.clone(),
                    file_size: item.item.file_size,
                    storage_type: item.item.storage_type.clone(),
                    content_type: item.item.content_type.clone(),
                    file_url,
                    add_time: item.item.file_user_add_time,
                    tags,
                    user_id: item.item.user_id,
                    add_user_id: item.item.add_user_id,
                    app_id: item.item.app_id,
                    status: item.item.status,
                    file_user_status: item.item.file_user_status,
                    source_url: item.item.source_url.clone(),
                    source_md5: item.item.source_md5.clone(),
                    modify_time: item.item.modify_time,
                    attr_local: item.attr_local.clone(),
                    attr_oss: item.attr_oss.clone(),
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
        total_param: &TotalParam,
    ) -> FileManagerResult<TotalRow> {
        let tag_name = format!("script_id_{}", script.id);

        Ok(self
            .file_dao
            .data_dao()
            .count_files_by_tag(&tag_name, None, app_id, total_param)
            .await?)
    }
}
