// 系统 RBAC 角色用户列表导出（管理员视角）
//
// CSV 列: id, role_id, user_id, timeout, status, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, RoleDataAttrParam, RoleDataParam};

use crate::dao::access::api::system::admin::CheckAdminRbacView;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_SYSTEM_RBAC_ROLE_USER: &str = "system_rbac_role_user";

pub struct SystemRbacRoleUserExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemRbacRoleUserExporter {
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
                .check(check_env, &CheckAdminRbacView {})
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
