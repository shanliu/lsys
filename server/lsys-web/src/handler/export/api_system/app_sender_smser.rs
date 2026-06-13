// 系统管理端短信消息列表导出
//
//   SystemSmserMessageListExporter — 系统所有短信消息列表
//     CSV 列: id, snid, area, mobile, try_num, status, add_time, send_time, setting_id

use std::path::PathBuf;
use std::sync::Arc;

use lsys_app_sender::dao::SmsSenderDao;
use lsys_app_sender::model::SenderSmsMessageStatus;
use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminSmsMgr;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporterCheck, WebExportCheckParam, WebResult};

pub const EXPORT_TYPE_SYSTEM_SMSER_MESSAGE_LIST: &str = "system_smser_message_list";

/// 系统短信消息列表权限检查器
pub struct SystemSmserMessageListExportCheck {
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporterCheck for SystemSmserMessageListExportCheck {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        _param: &WebExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(check_env, &CheckAdminSmsMgr {})
            .await?;
        Ok(())
    }
}

/// 系统短信消息列表导出器
pub struct SystemSmserMessageListExporter {
    pub smser_dao: Arc<SmsSenderDao>,
}

impl Exporter<crate::dao::WebError> for SystemSmserMessageListExporter {
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
            let tpl_key = params["tpl_key"].as_str();
            let body_id = params["body_id"].as_u64();
            let snid = params["snid"].as_u64();
            let status = params["status"]
                .as_i64()
                .and_then(|v| SenderSmsMessageStatus::try_from(v as i8).ok());
            let mobile = params["mobile"].as_str();

            let fluent = fluent_mgr.locale(lang.as_deref());
            let mut w = CsvWriter::new(&record)
                .header(export_header!(
                    fluent,
                    EXPORT_TYPE_SYSTEM_SMSER_MESSAGE_LIST,
                    "id",
                    "snid",
                    "area",
                    "mobile",
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
                    .smser_dao
                    .sms_record
                    .message_list(
                        Some(0),
                        Some(0),
                        tpl_key,
                        body_id,
                        snid,
                        status,
                        mobile,
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
                            msg.area.clone(),
                            msg.mobile.clone(),
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
