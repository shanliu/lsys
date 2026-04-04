// 邮件发送相关导出
//
// 包含三个导出器：
//   MailerMessageListExporter — 邮件消息列表
//     CSV 列: id, snid, to_mail, try_num, status, add_time, send_time, setting_id
//
//   MailerMessageLogExporter — 邮件发送日志
//     CSV 列: id, send_data, res_data, status, add_time
//
//   MailerTplConfigExporter — 邮件模板配置列表
//     CSV 列: id, app_id, name, tpl_key, setting_id, status, user_id, change_user_id, change_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app_sender::dao::MailSenderDao;
use lsys_app_sender::model::SenderMailMessageStatus;
use lsys_core::db::{
    CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort, OffsetPageParam,
    OffsetPageValue,
};

use crate::dao::access::api::system::user::{
    CheckUserAppSenderMailConfig, CheckUserAppSenderMailView,
};
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::WebError;
use crate::dao::WebRbac;
use crate::dao::WebResult;
use crate::model::ExportTaskModel;

pub const EXPORT_TYPE_USER_MAILER_MESSAGE_LIST: &str = "user_mailer_message_list";
pub const EXPORT_TYPE_USER_MAILER_MESSAGE_LOG: &str = "user_mailer_message_log";
pub const EXPORT_TYPE_USER_MAILER_TPL_CONFIG: &str = "user_mailer_tpl_config";

/// 邮件消息列表导出
pub struct MailerMessageListExporter {
    pub mailer_dao: Arc<MailSenderDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for MailerMessageListExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac
                .check(
                    check_env,
                    &CheckUserAppSenderMailView {
                        res_user_id: user_id,
                    },
                )
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
            let user_id = params["user_id"].as_u64();
            let app_id = params["app_id"].as_u64();
            let tpl_key = params["tpl_key"].as_str();
            let body_id = params["body_id"].as_u64();
            let snid = params["snid"].as_u64();
            let status = params["status"]
                .as_i64()
                .and_then(|v| SenderMailMessageStatus::try_from(v as i8).ok());
            let to_mail = params["to_mail"].as_str();

            let mut w = CsvWriter::new(&record)
                .header((
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

            w.finish().await
        })
    }
}

/// 邮件发送日志导出
pub struct MailerMessageLogExporter {
    pub mailer_dao: Arc<MailSenderDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for MailerMessageLogExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac
                .check(
                    check_env,
                    &CheckUserAppSenderMailView {
                        res_user_id: user_id,
                    },
                )
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
            let message_id = params["message_id"].as_u64().unwrap_or(0);

            let mut w = CsvWriter::new(&record)
                .header(("id", "executor_type", "message", "status", "create_time"))
                .await?;

            let total = self
                .mailer_dao
                .mail_record
                .message_log_count(message_id)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .mailer_dao
                    .mail_record
                    .message_log_list(message_id, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.id,
                            item.executor_type.clone(),
                            item.message.clone(),
                            item.status,
                            item.create_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}

/// 邮件模板配置列表导出
pub struct MailerTplConfigExporter {
    pub mailer_dao: Arc<MailSenderDao>,
    pub web_rbac: Arc<WebRbac>,
}

impl Exporter for MailerTplConfigExporter {
    fn check<'a>(
        &'a self,
        check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async move {
            self.web_rbac
                .check(
                    check_env,
                    &CheckUserAppSenderMailConfig {
                        res_user_id: user_id,
                    },
                )
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
            let id = params["id"].as_u64();
            let user_id = params["user_id"].as_u64();
            let app_id = params["app_id"].as_u64();
            let tpl = params["tpl"].as_str();

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "app_id",
                    "name",
                    "tpl_key",
                    "setting_id",
                    "status",
                    "user_id",
                    "change_user_id",
                    "change_time",
                ))
                .await?;

            let total = self
                .mailer_dao
                .tpl_config
                .count_config(id, user_id, app_id, tpl, None)
                .await? as u64;
            if total > 0 {
                let page = OffsetPageParam::new(Some(OffsetPageValue::new(0, total)));
                let items = self
                    .mailer_dao
                    .tpl_config
                    .list_config(id, user_id, app_id, tpl, None, &page)
                    .await?;
                let rows: Vec<_> = items
                    .iter()
                    .map(|(cfg, _setting)| {
                        (
                            cfg.id,
                            cfg.app_id,
                            cfg.name.clone(),
                            cfg.tpl_key.clone(),
                            cfg.setting_id,
                            cfg.status,
                            cfg.user_id,
                            cfg.change_user_id,
                            cfg.change_time,
                        )
                    })
                    .collect();
                w.write_batch(rows).await?;
            }

            w.finish().await
        })
    }
}
