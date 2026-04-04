// 系统 RBAC 资源类型列表导出（管理员视角）
//
// 包含两个导出器：
//   SystemRbacResTypeExporter — 资源类型列表
//     CSV 列: user_id, app_id, res_type, res_total
//
//   SystemRbacResTypeOpExporter — 资源类型关联的操作列表
//     CSV 列: id, op_id, res_type, user_id, app_id, status, change_time, change_user_id

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use lsys_rbac::dao::{RbacDao, ResTypeListParam, ResTypeParam};

use crate::dao::access::api::system::admin::CheckAdminRbacView;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE: &str = "system_rbac_res_type";
pub const EXPORT_TYPE_SYSTEM_RBAC_RES_TYPE_OP: &str = "system_rbac_res_type_op";

/// 系统资源类型列表导出
pub struct SystemRbacResTypeExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemRbacResTypeExporter {
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
            let res_type = params["res_type"].as_str();

            let res_param = ResTypeListParam {
                user_id: Some(0),
                app_id: Some(0),
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

/// 系统资源类型操作列表导出
pub struct SystemRbacResTypeOpExporter {
    pub rbac_dao: Arc<RbacDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for SystemRbacResTypeOpExporter {
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
            let res_type_str = params["res_type"].as_str().unwrap_or("").to_string();

            let res_type_param = ResTypeParam {
                res_type: &res_type_str,
                user_id: 0,
                app_id: 0,
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
