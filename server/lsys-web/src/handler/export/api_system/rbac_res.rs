// 系统 RBAC 资源列表导出（管理员视角）
//
// CSV 列: id, res_type, res_data, user_id, app_id, res_name, status, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, ResDataAttrParam, ResDataParam};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminRbacView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_SYSTEM_RBAC_RES: &str = "system_rbac_res";

pub struct SystemRbacResExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for SystemRbacResExporter {
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

impl Exporter<crate::dao::WebError> for SystemRbacResExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let user_id = params["user_id"].as_u64().or(Some(0));
            let app_id = params["app_id"].as_u64().or(Some(0));
            let res_type = params["res_type"].as_str();
            let res_data = params["res_data"].as_str();
            let res_name = params["res_name"].as_str();

            let res_param = ResDataParam {
                user_id,
                app_id,
                res_type,
                res_data,
                res_name,
                ids: None,
            };
            let res_attr = ResDataAttrParam {
                op_count: false,
                perm_count: false,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "res_type",
                    "res_data",
                    "user_id",
                    "app_id",
                    "res_name",
                    "status",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self.rbac_dao.res.res_count(&res_param).await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .res
                    .res_info(&res_param, &res_attr, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(res, _info)| {
                        (
                            res.id,
                            res.res_type.clone(),
                            res.res_data.clone(),
                            res.user_id,
                            res.app_id,
                            res.res_name.clone(),
                            res.status,
                            res.change_user_id,
                            res.change_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
