// 登录历史导出（仅用户自身）
//
//   UserLoginHistoryExporter — 用户自身的登录历史
//     CSV 列: id, login_type, login_account, login_ip, login_city, account_id, is_login, login_msg, add_time

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_user::dao::AccountDao;

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserInfoEdit;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_USER_LOGIN_HISTORY: &str = "user_login_history";
pub struct UserLoginHistoryExporter {
    pub account_dao: Arc<AccountDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for UserLoginHistoryExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &crate::dao::ExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(
                check_env,
                &CheckUserInfoEdit {
                    res_user_id: param.user_id,
                },
            )
            .await?;
        Ok(())
    }
}

impl Exporter<crate::dao::WebError> for UserLoginHistoryExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let account_id = params["account_id"].as_u64();
            let login_type = params["login_type"].as_str();
            let login_account = params["login_account"].as_str();
            let login_ip = params["login_ip"].as_str();
            let is_login = params["is_login"].as_i64().map(|v| v as i8);

            let mut w = CsvWriter::new(&record)
                .header((
                    "id",
                    "login_type",
                    "login_account",
                    "login_ip",
                    "login_city",
                    "account_id",
                    "is_login",
                    "login_msg",
                    "add_time",
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
                    .account_dao
                    .account_login_history
                    .history_data(
                        account_id,
                        login_account,
                        is_login,
                        login_type,
                        login_ip,
                        &page_param,
                    )
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        (
                            item.id,
                            item.login_type.clone(),
                            item.login_account.clone(),
                            item.login_ip.clone(),
                            item.login_city.clone(),
                            item.account_id,
                            item.is_login,
                            item.login_msg.clone(),
                            item.add_time,
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
