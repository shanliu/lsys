// 系统可用角色用户搜索导出（管理员视角）
//
// CSV 列: id, app_id, user_data, user_account, user_nickname

use std::path::PathBuf;
use std::sync::Arc;

use lsys_access::dao::{AccessDao, UserDataParam};
use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};

use crate::dao::access::api::system::admin::CheckAdminRbacEdit;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_SYSTEM_ROLE_USER_AVAILABLE: &str = "system_role_user_available";

pub struct SystemRoleUserAvailableExporter {
    pub access_dao: Arc<AccessDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemRoleUserAvailableExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>,
    > {
        Box::pin(async move {
            self.web_rbac
                .check(check_env, &CheckAdminRbacEdit {})
                .await?;
            Ok(())
        })
    }

    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PathBuf, WebError>> + Send + 'a>>
    {
        Box::pin(async move {
            let user_data = params["user_data"].as_str();

            let user_param = UserDataParam {
                app_id: Some(0),
                user_data: None,
                user_account: None,
                user_any: user_data,
            };

            let mut w = CsvWriter::new(&record)
                .header(("id", "app_id", "user_data", "user_account", "user_nickname"))
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
                    .user_data(&user_param, &page_param)
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
                            item.user_data.clone(),
                            item.user_account.clone(),
                            item.user_nickname.clone(),
                        )
                    })
                    .collect();

                w.write_batch(rows).await?;

                cursor = page_data.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }

            w.finish().await
        })
    }
}
