// RBAC 角色列表导出
//

//
// CSV 列: id, app_id, role_key, role_name, user_range, res_range, status, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, RoleDataAttrParam, RoleDataParam};

use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::dao::WebApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::{CheckUserAppView, CheckUserRbacView};
use crate::dao::export_task::writer::CsvWriter;
use crate::model::ExportTaskModel;
use crate::dao::export_task::exporter::Exporter;

pub const EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE: &str = "user_rbac_system_role";
pub const EXPORT_TYPE_USER_RBAC_APP_ROLE: &str = "user_rbac_app_role";

pub struct RbacRoleExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for RbacRoleExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>>
    {
        Box::pin(async move {
            if export_type == EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE {
                self.web_rbac
                    .check(check_env, &CheckUserRbacView { res_user_id: user_id })
                    .await?
            } else {
                if app_id == 0 {
                    return Err(WebError::Message(lsys_core::fluent_message!("export-miss-app-id")));
                }
                let app = self.web_app.app_dao.app.find_by_id(app_id).await?;
                self.web_rbac
                    .check(check_env, &CheckUserAppView { res_user_id: app.user_id })
                    .await?;
                app.app_status_check()?;
                self.web_app.app_dao.app.inner_feature_sub_app_check(&app).await?;
            }
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

            let user_id = params["user_id"].as_u64().unwrap_or(0);
            let app_id = params["app_id"].as_u64();
            let role_key = params["role_key"].as_str();
            let role_name = params["role_name"].as_str();
            let user_range = params["user_range"]
                .as_i64()
                .and_then(|v| lsys_rbac::model::RbacRoleUserRange::try_from(v as i8).ok());
            let res_range = params["res_range"]
                .as_i64()
                .and_then(|v| lsys_rbac::model::RbacRoleResRange::try_from(v as i8).ok());

            let role_param = RoleDataParam {
                user_id,
                app_id,
                user_range,
                res_range,
                role_key,
                role_name,
                ids: None,
            };
            let role_attr = RoleDataAttrParam {
                user_count: None,
                user_data: None,
                res_count: None,
                res_op_count: None,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id", "app_id", "role_key", "role_name",
                    "user_range", "res_range", "status", "change_user_id", "change_time",
                ))
                .await?;

            let total = self
                .rbac_dao
                .role
                .role_count(&role_param)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .role
                    .role_info(&role_param, &role_attr, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(role, _info)| (
                        role.id,
                        role.app_id,
                        role.role_key.clone(),
                        role.role_name.clone(),
                        role.user_range,
                        role.res_range,
                        role.status,
                        role.change_user_id,
                        role.change_time,
                    ))
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
