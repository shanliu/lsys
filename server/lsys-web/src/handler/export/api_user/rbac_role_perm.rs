// RBAC 角色权限列表导出
//

//
// CSV 列: op_id, op_key, op_name, res_id, res_type, res_data, res_name, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, RoleDataAttrParam, RoleDataParam};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserRbacView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, FileManagerError, WebExporter, WebResult};

pub const EXPORT_TYPE_USER_RBAC_SYSTEM_ROLE_PERM: &str = "user_rbac_system_role_perm";
pub const EXPORT_TYPE_USER_RBAC_APP_ROLE_PERM: &str = "user_rbac_app_role_perm";

pub struct RbacRolePermExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for RbacRolePermExporter {
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

impl Exporter<crate::dao::WebError> for RbacRolePermExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let role_id = params["role_id"].as_u64().unwrap_or(0);

            // 先加载角色模型
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

            let (role, _) = roles.into_iter().next().ok_or_else(|| {
                FileManagerError::Message(lsys_core::fluent_message!("role-not-found"))
            })?;

            let mut w = CsvWriter::new(&record)
                .header((
                    "op_id",
                    "op_key",
                    "op_name",
                    "res_id",
                    "res_type",
                    "res_data",
                    "res_name",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.rbac_dao.role.role_perm_count(&role).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self.rbac_dao.role.role_perm_data(&role, &page).await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.op_id,
                            item.op_key.clone(),
                            item.op_name.clone(),
                            item.res_id,
                            item.res_type.clone(),
                            item.res_data.clone(),
                            item.res_name.clone(),
                            item.change_user_id,
                            item.change_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
