// 用户 APP RBAC 资源列表导出
//
//   AppResDataExporter — 对指定 APP 的资源列表
//     CSV 列: id, user_id, res_type, res_data, res_name, change_time, op_count, perm_count
//   AppResDataExportCheck — 权限检查器

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, ResDataAttrParam, ResDataParam};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserRbacView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_APP_RES_DATA: &str = "app_res_data";

/// APP 资源数据权限检查器
pub struct AppResDataExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for AppResDataExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &WebExportCheckParam<'_>,
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

/// APP 资源数据导出器
pub struct AppResDataExporter {
    pub rbac_dao: Arc<RbacDao>,
}

impl Exporter<crate::dao::WebError> for AppResDataExporter {
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
            let user_id = Some(record.user_id);
            let app_id = params["app_id"].as_u64();
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
                op_count: true,
                perm_count: true,
            };

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_APP_RES_DATA,
                    "id",
                    "user_id",
                    "res_type",
                    "res_data",
                    "res_name",
                    "change_time",
                    "op_count",
                    "perm_count",
                ))
                .await?;

            let total = self.rbac_dao.res.res_count(&res_param).await? as u64;
            if total > 0 {
                let page =
                    OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .res
                    .res_info(&res_param, &res_attr, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(e, info)| {
                        (
                            e.id,
                            e.user_id,
                            e.res_type.clone(),
                            e.res_data.clone(),
                            e.res_name.clone(),
                            e.change_time,
                            info.op_count,
                            info.perm_count,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
