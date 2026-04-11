// 系统 RBAC 操作列表导出（管理员视角）
//
// CSV 列: id, app_id, op_key, op_name, status, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{OpDataAttrParam, OpDataParam, RbacDao};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminRbacView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_SYSTEM_RBAC_OP: &str = "system_rbac_op";

pub struct SystemRbacOpExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for SystemRbacOpExporter {
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

impl Exporter<crate::dao::WebError> for SystemRbacOpExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let op_name = params["op_name"].as_str();
            let op_key = params["op_key"].as_str();

            let op_param = OpDataParam {
                user_id: 0,
                app_id: None,
                op_name,
                op_key,
                ids: None,
            };
            let op_attr = OpDataAttrParam {
                res_type_count: false,
                check_role_use: false,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "app_id",
                    "op_key",
                    "op_name",
                    "status",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.rbac_dao.op.op_count(&op_param).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self.rbac_dao.op.op_info(&op_param, &op_attr, &page).await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(op, _info)| {
                        (
                            op.id,
                            op.app_id,
                            op.op_key.clone(),
                            op.op_name.clone(),
                            op.status,
                            op.change_user_id,
                            op.change_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
