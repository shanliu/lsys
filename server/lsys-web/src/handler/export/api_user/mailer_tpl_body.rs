// 邮件模板内容列表导出
//

//
// CSV 列: id, app_id, tpl_id, sender_type, status, add_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app_sender::dao::MessageTpls;
use lsys_app_sender::model::SenderType;
use lsys_core::db::{OffsetPageParam, OffsetPageValue};

use crate::dao::WebError;
use crate::dao::WebResult;
use crate::dao::WebRbac;
use crate::dao::WebApp;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserAppSenderMailConfig;
use crate::dao::export_task::writer::CsvWriter;
use crate::model::ExportTaskModel;
use crate::dao::export_task::exporter::Exporter;
use crate::handler::APP_FEATURE_MAIL;

pub const EXPORT_TYPE_USER_MAILER_TPL_BODY: &str = "user_mailer_tpl_body";

pub struct MailerTplBodyExporter {
    pub tpl_dao: Arc<MessageTpls>,
    pub web_rbac: Arc<WebRbac>,
    pub web_app: Arc<WebApp>,
}

impl Exporter for MailerTplBodyExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>>
    {
        Box::pin(async move {
            if app_id == 0 {
                return Err(WebError::Message(lsys_core::fluent_message!("export-miss-app-id")));
            }
            self.web_rbac
                .check(check_env, &CheckUserAppSenderMailConfig { res_user_id: user_id })
                .await?;
            let app = self.web_app.app_dao.app.find_by_id(app_id).await?;
            app.app_status_check()?;
            self.web_app.app_dao.app.cache().exter_feature_check(&app, &[APP_FEATURE_MAIL]).await?;
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

            let app_id = params["app_id"].as_u64().unwrap_or(0);
            let id = params["id"].as_u64();
            let tpl_id = params["tpl_id"].as_str();
            let tpl_id_like = params["tpl_id_like"].as_str();

            let mut w = CsvWriter::new(&record)
                .header(("id", "app_id", "tpl_id", "sender_type", "status", "change_time"))
                .await?;

            let total = self
                .tpl_dao
                .list_count(
                    app_id,
                    Some(SenderType::Mailer),
                    id,
                    tpl_id,
                    tpl_id_like,
                )
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .tpl_dao
                    .list_data(
                        app_id,
                        Some(SenderType::Mailer),
                        id,
                        tpl_id,
                        tpl_id_like,
                        &page,
                    )
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| (
                        item.id,
                        item.app_id,
                        item.tpl_id.clone(),
                        item.sender_type,
                        item.status,
                        item.change_time,
                    ))
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
