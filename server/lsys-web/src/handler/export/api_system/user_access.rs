// 系统管理员用户登录历史导出
//
//   UserAccessExporter — 用户登录历史列表
//     CSV 列: id, app_id, oauth_app_id, user_id, token_data, login_time, last_access_time, is_enable

use std::path::PathBuf;
use std::sync::Arc;

use lsys_access::dao::{AccessDao, SessionDataParam};
use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminUserManage;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_SYSTEM_USER_ACCESS: &str = "system_user_access";

/// 系统管理员用户登录历史权限检查器
pub struct UserAccessExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for UserAccessExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        _param: &WebExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(check_env, &CheckAdminUserManage {})
            .await?;
        Ok(())
    }
}

/// 系统管理员用户登录历史导出器
pub struct UserAccessExporter {
    pub access_dao: Arc<AccessDao>,
}

impl Exporter<crate::dao::WebError> for UserAccessExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
        lang: Option<String>,
        fluent_mgr: Arc<lsys_core::fluents::FluentMgr>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let app_id = params["app_id"].as_u64();
            let oauth_app_id = params["oauth_app_id"].as_u64();
            let user_id = params["user_id"].as_u64();
            let is_enable = params["is_enable"].as_bool();

            let session_param = SessionDataParam {
                app_id,
                oauth_app_id,
                user_id,
                is_enable,
            };

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_SYSTEM_USER_ACCESS,
                    "id",
                    "app_id",
                    "oauth_app_id",
                    "user_id",
                    "token_data",
                    "login_type",
                    "login_ip",
                    "status",
                    "add_time",
                    "expire_time",
                    "logout_time",
                ))
                .await?;

            let mut cursor: Option<u64> = None;
            loop {
                let page_param = CursorPageParam::new(
                    CursorPageDir::Next,
                    CursorConfig::primary(CursorPageSort::Desc),
                    cursor,
                    CursorLimit::Limit {
                        limit: 200,
                        more: true,
                    },
                );

                let (items, page_data) = self
                    .access_dao
                    .user
                    .session_data(&session_param, &page_param)
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.id,
                            item.app_id,
                            item.oauth_app_id,
                            item.user_id,
                            item.token_data.clone(),
                            item.login_type.clone(),
                            item.login_ip.clone(),
                            item.status,
                            item.add_time,
                            item.expire_time,
                            item.logout_time,
                        )
                    })
                    .collect();

                w.write_batch(rows).await?;

                cursor = page_data.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
