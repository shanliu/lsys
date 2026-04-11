// 系统 RBAC 角色列表导出（管理员视角）
//
// CSV 列: id, app_id, role_key, role_name, user_range, res_range, status, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, RoleDataAttrParam, RoleDataParam};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminRbacView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_SYSTEM_RBAC_ROLE: &str = "system_rbac_role";

pub struct SystemRbacRoleExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for SystemRbacRoleExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        _param: &crate::dao::ExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(check_env, &CheckAdminRbacView {})
            .await?;
        Ok(())
    }
}

impl Exporter<crate::dao::WebError> for SystemRbacRoleExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let role_key = params["role_key"].as_str();
            let role_name = params["role_name"].as_str();
            let user_range = params["user_range"]
                .as_i64()
                .and_then(|v| lsys_rbac::model::RbacRoleUserRange::try_from(v as i8).ok());
            let res_range = params["res_range"]
                .as_i64()
                .and_then(|v| lsys_rbac::model::RbacRoleResRange::try_from(v as i8).ok());

            let role_param = RoleDataParam {
                user_id: 0,
                app_id: Some(0),
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
                    "id",
                    "app_id",
                    "role_key",
                    "role_name",
                    "user_range",
                    "res_range",
                    "status",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.rbac_dao.role.role_count(&role_param).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .role
                    .role_info(&role_param, &role_attr, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(role, _info)| {
                        (
                            role.id,
                            role.app_id,
                            role.role_key.clone(),
                            role.role_name.clone(),
                            role.user_range,
                            role.res_range,
                            role.status,
                            role.change_user_id,
                            role.change_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
