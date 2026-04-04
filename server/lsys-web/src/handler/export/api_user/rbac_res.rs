// RBAC 资源列表导出
//

//
// CSV 列: id, res_type, res_data, user_id, app_id, res_name, status, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, ResDataAttrParam, ResDataParam};

use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::dao::WebApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserAppView;
use crate::dao::export_task::writer::CsvWriter;
use crate::model::ExportTaskModel;
use crate::dao::export_task::exporter::Exporter;

pub const EXPORT_TYPE_USER_RBAC_APP_RES: &str = "user_rbac_app_res";

pub struct RbacResExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for RbacResExporter {
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
                .check(check_env, &CheckUserAppView { res_user_id: app.user_id })
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

            let user_id = params["user_id"].as_u64();
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
                op_count: false,
                perm_count: false,
            };

            let mut w = CsvWriter::new(&record)
                .header((
                    "id", "res_type", "res_data", "user_id", "app_id",
                    "res_name", "status", "change_user_id", "change_time",
                ))
                .await?;

            let total = self
                .rbac_dao
                .res
                .res_count(&res_param)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .res
                    .res_info(&res_param, &res_attr, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(res, _info)| (
                        res.id,
                        res.res_type.clone(),
                        res.res_data.clone(),
                        res.user_id,
                        res.app_id,
                        res.res_name.clone(),
                        res.status,
                        res.change_user_id,
                        res.change_time,
                    ))
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
