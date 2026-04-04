// RBAC 资源类型列表导出
//
// 包含两个导出器：
//   RbacResTypeExporter — 资源类型列表
//     CSV 列: user_id, app_id, res_type, res_total
//
//   RbacResTypeOpExporter — 资源类型关联的操作列表
//     CSV 列: id, op_id, res_type, user_id, app_id, status, change_time, change_user_id

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, ResTypeListParam, ResTypeParam};

use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::dao::WebApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::{CheckUserAppEdit, CheckUserAppView};
use crate::dao::export_task::writer::CsvWriter;
use crate::model::ExportTaskModel;
use crate::dao::export_task::exporter::Exporter;

pub const EXPORT_TYPE_USER_RBAC_APP_RES_TYPE: &str = "user_rbac_app_res_type";
pub const EXPORT_TYPE_USER_RBAC_APP_RES_TYPE_OP: &str = "user_rbac_app_res_type_op";

/// 资源类型列表导出
pub struct RbacResTypeExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for RbacResTypeExporter {
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

            let res_param = ResTypeListParam {
                user_id,
                app_id,
                res_type,
            };

            let mut w = CsvWriter::new(&record)
                .header(("user_id", "app_id", "res_type", "res_total"))
                .await?;

            let total = self
                .rbac_dao
                .res
                .res_type_count(&res_param)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .res
                    .res_type_data(&res_param, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| (
                        item.user_id,
                        item.app_id,
                        item.res_type.clone(),
                        item.res_total,
                    ))
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}

/// 资源类型操作列表导出
pub struct RbacResTypeOpExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for RbacResTypeOpExporter {
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

            let res_type_str = params["res_type"].as_str().unwrap_or("").to_string();
            let user_id = params["user_id"].as_u64().unwrap_or(0);
            let app_id = params["app_id"].as_u64().unwrap_or(0);

            let res_type_param = ResTypeParam {
                res_type: &res_type_str,
                user_id,
                app_id,
            };

            let mut w = CsvWriter::new(&record)
                .header(("id", "op_id", "res_type", "user_id", "app_id", "status", "change_time", "change_user_id"))
                .await?;

            let total = self
                .rbac_dao
                .res
                .res_type_op_count(&res_type_param)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .rbac_dao
                    .res
                    .res_type_op_data(&res_type_param, None, false, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| (
                        item.op_res.id,
                        item.op_res.op_id,
                        item.op_res.res_type.clone(),
                        item.op_res.user_id,
                        item.op_res.app_id,
                        item.op_res.status,
                        item.op_res.change_time,
                        item.op_res.change_user_id,
                    ))
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
