// 系统用户搜索导出（管理员视角）
//
// CSV 列: account_id, cats

use std::path::PathBuf;
use std::sync::Arc;

use lsys_core::db::{CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort};
use lsys_user::dao::AccountDao;

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminUserManage;
use crate::dao::export_task::exporter::Exporter;
use crate::dao::export_task::writer::CsvWriter;
use crate::dao::{ExportTaskModel, WebExporter, WebResult};

pub const EXPORT_TYPE_SYSTEM_ACCOUNT_SEARCH: &str = "system_account_search";

pub struct SystemAccountSearchExporter {
    pub account_dao: Arc<AccountDao>,
    pub web_rbac: Arc<crate::dao::WebRbac>,
}

#[async_trait::async_trait]
impl WebExporter for SystemAccountSearchExporter {
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        _param: &crate::dao::ExportCheckParam<'_>,
    ) -> WebResult<()> {
        self.web_rbac
            .check(check_env, &CheckAdminUserManage {})
            .await?;
        Ok(())
    }
}

impl Exporter<crate::dao::WebError> for SystemAccountSearchExporter {
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, crate::dao::WebError>> + Send + 'a>,
    > {
        Box::pin(async move {
            let key_word = params["key_word"].as_str().unwrap_or("").to_string();
            let enable = params["enable"].as_bool().unwrap_or(true);

            let mut w = CsvWriter::new(&record)
                .header(("account_id", "cats"))
                .await?;

            let mut cursor: Option<u64> = None;
            loop {
                let page_param = CursorPageParam::new(
                    CursorPageDir::Next,
                    CursorConfig::primary(CursorPageSort::Desc),
                    cursor,
                    CursorLimit::Limit {
                        limit: 200,
                        more: true,
                    },
                );

                let (items, page_data) = self
                    .account_dao
                    .account
                    .search(&key_word, enable, &page_param)
                    .await?;

                if items.is_empty() {
                    break;
                }

                let rows: Vec<_> = items
                    .iter()
                    .map(|item| {
                        let cats_str = item
                            .cats
                            .iter()
                            .map(|(k, v)| format!("{}:{}", *k as i8, v))
                            .collect::<Vec<_>>()
                            .join(",");
                        (item.account_id, cats_str)
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
