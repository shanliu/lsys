// RBAC 可用角色用户搜索导出
//

//           /api/user/rbac/system/role_user_available,
//           /api/user/rbac/app/role_user_available
//
// CSV 列: id, app_id, user_data, user_account, user_nickname

use std::path::PathBuf;
use std::sync::Arc;

use lsys_access::dao::{AccessDao, UserDataParam};
use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserRbacView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_USER_SYSTEM_ROLE_USER_AVAILABLE: &str = "user_system_role_user_available";
pub const EXPORT_TYPE_USER_APP_ROLE_USER_AVAILABLE: &str = "user_app_role_user_available";

pub struct RoleUserAvailableExporter {
    pub access_dao: Arc<AccessDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for RoleUserAvailableExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &crate::dao::ExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserRbacView {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

impl Exporter<crate::dao::WebError> for RoleUserAvailableExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let user_data = params["user_data"].as_str();
            let app_id = params["app_id"].as_u64().unwrap_or(0);

            let user_param = UserDataParam {
                app_id: Some(app_id),
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

            w.finish().await.map_err(Into::into)
        })
    }
}
