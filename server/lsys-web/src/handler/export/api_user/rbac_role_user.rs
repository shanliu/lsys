// RBAC 角色用户列表导出
//

//
// CSV 列: id, role_id, user_id, timeout, status, change_user_id, change_time

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

pub const EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_USER: &str = "user_rbac_system_role_user";
pub const EXPORT_TYPE_USER_RBAC_APP_ROLE_USER: &str = "user_rbac_app_role_user";

pub struct RbacRoleUserExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for RbacRoleUserExporter {
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
            if export_type == EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_USER {
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

            let role_id = params["role_id"].as_u64().unwrap_or(0);
            let all = params["all"].as_bool().unwrap_or(false);

            // 先通过 role_info 加载角色模型
            let role_param = RoleDataParam {
                user_id: 0,
                app_id: None,
                user_range: None,
                res_range: None,
                role_key: None,
                role_name: None,
                ids: Some(&[role_id]),
            };
            let role_attr = RoleDataAttrParam {
                user_count: None,
                user_data: None,
                res_count: None,
                res_op_count: None,
            };
            let single_page = OffsetPageParam::new(Some(OffsetPageValue::new(0, 1)));
            let roles = self
                .rbac_dao
                .role
                .role_info(&role_param, &role_attr, &single_page)
                .await?;

            let (role, _) = roles
                .into_iter()
                .next()
                .ok_or_else(|| WebError::Message(lsys_core::fluent_message!("role-not-found", format!("role {} not found", role_id))))?;

            let mut w = CsvWriter::new(&record)
                .header(("id", "role_id", "user_id", "timeout", "status", "change_user_id", "change_time"))
                .await?;

            let total = self
                .rbac_dao
                .role
                .role_user_count(&role, all)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .role
                    .role_user_data(&role, all, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| (
                        item.id,
                        item.role_id,
                        item.user_id,
                        item.timeout,
                        item.status,
                        item.change_user_id,
                        item.change_time,
                    ))
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
