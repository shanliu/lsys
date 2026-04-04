// RBAC 操作列表导出
//

//
// CSV 列: id, app_id, op_key, op_name, status, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{OpDataAttrParam, OpDataParam, RbacDao};

use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::dao::WebApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserAppEdit;
use crate::dao::export_task::writer::CsvWriter;
use crate::model::ExportTaskModel;
use crate::dao::export_task::exporter::Exporter;

pub const EXPORT_TYPE_USER_RBAC_APP_OP: &str = "user_rbac_app_op";

pub struct RbacOpExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for RbacOpExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>>
    {
        Box::pin(async move {
            if app_id == 0 {
                return Err(WebError::Message(lsys_core::fluent_message!("export-miss-app-id")));
            }
            let app = self.web_app.app_dao.app.find_by_id(app_id).await?;
            self.web_rbac
                .check(check_env, &CheckUserAppEdit { res_user_id: app.user_id })
                .await?;
            app.app_status_check()?;
            self.web_app.app_dao.app.inner_feature_sub_app_check(&app).await?;
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
            let op_name = params["op_name"].as_str();
            let op_key = params["op_key"].as_str();

            let op_param = OpDataParam {
                user_id,
                app_id,
                op_name,
                op_key,
                ids: None,
            };
            let op_attr = OpDataAttrParam {
                res_type_count: false,
                check_role_use: false,
            };

            let mut w = CsvWriter::new(&record)
                .header(("id", "app_id", "op_key", "op_name", "status", "change_user_id", "change_time"))
                .await?;

            let total = self
                .rbac_dao
                .op
                .op_count(&op_param)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .op
                    .op_info(&op_param, &op_attr, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(op, _info)| (
                        op.id,
                        op.app_id,
                        op.op_key.clone(),
                        op.op_name.clone(),
                        op.status,
                        op.change_user_id,
                        op.change_time,
                    ))
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
