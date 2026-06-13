// 邮件发送相关导出
//
//   MailerMessageListExporter — 邮件消息列表
//     CSV 列: id, snid, to_mail, try_num, status, add_time, send_time, setting_id
//   MailerMessageListExportCheck — 权限检查器

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app_sender::dao::MailSenderDao;
use lsys_app_sender::model::SenderMailMessageStatus;
use lsys_core::db::{
    CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort,
};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserAppSenderMailView;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_USER_MAILER_MESSAGE_LIST: &str = "user_mailer_message_list";

/// 邮件消息列表权限检查器
pub struct MailerMessageListExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for MailerMessageListExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &WebExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserAppSenderMailView {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

/// 邮件消息列表导出器
pub struct MailerMessageListExporter {
    pub mailer_dao: Arc<MailSenderDao>,
}

impl Exporter<crate::dao::WebError> for MailerMessageListExporter {
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
            let tpl_key = params["tpl_key"].as_str();
            let body_id = params["body_id"].as_u64();
            let snid = params["snid"].as_u64();
            let status = params["status"]
                .as_i64()
                .and_then(|v| SenderMailMessageStatus::try_from(v as i8).ok());
            let to_mail = params["to_mail"].as_str();

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_USER_MAILER_MESSAGE_LIST,
                    "id",
                    "snid",
                    "to_mail",
                    "try_num",
                    "status",
                    "add_time",
                    "send_time",
                    "setting_id",
                ))
                .await?;

            let mut cursor: Option<u64> = None;
            loop {
                let page_param = CursorPageParam::new(
                    CursorPageDir::Next,
                    CursorConfig::primary(CursorPageSort::Asc),
                    cursor,
                    CursorLimit::Limit {
                        limit: 200,
                        more: true,
                    },
                );

                let (items, page_data) = self
                    .mailer_dao
                    .mail_record
                    .message_list(
                        user_id,
                        app_id,
                        tpl_key,
                        body_id,
                        snid,
                        status,
                        to_mail,
                        &page_param,
                    )
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|(msg, _body)| {
                        (
                            msg.id,
                            msg.snid,
                            msg.to_mail.clone(),
                            msg.try_num,
                            msg.status,
                            msg.add_time,
                            msg.send_time,
                            msg.setting_id,
                        )
                    })
                    .collect();

                w.write_batch(rows).await?;

                cursor = page_data.next_cursor;
                if cursor.is_none() {
                    break;
                }
            }

            w.finish().await.map_err(Into::into)
        })
    }
}
