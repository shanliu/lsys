use lsys_access::dao::AccessSession;
use lsys_app_barcode::dao::BarcodeParseRecord;

use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::user::CheckUserBarCodeEdit;
use crate::dao::access::api::user::CheckUserBarCodeView;

#[derive(Debug, Deserialize)]
pub struct ParseRecordListParam {
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn parse_record_list(
    param: &ParseRecordListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserBarCodeView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .list_parse_record(
            auth_data.user_id(),
            param.app_id,
            param.barcode_type.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_barcode
                .barcode_dao
                .count_parse_record(
                    auth_data.user_id(),
                    param.app_id,
                    param.barcode_type.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };
    let data = data
        .into_iter()
        .map(|tmp| match tmp {
            BarcodeParseRecord::Succ(t) => {
                json!({
                    "id":t.0.id,
                    "app_id":t.0.app_id,
                    "bar_type":t.0.barcode_type,
                    "status":1,
                    "text":t.1.text,
                    "error":"",
                    "hash":t.0.file_hash,
                    "create_time":t.0.create_time
                })
            }
            BarcodeParseRecord::Fail(t) => {
                json!({
                    "id":t.id,
                    "app_id":t.app_id,
                    "bar_type":t.barcode_type,
                    "text":"",
                    "status":0,
                    "error":t.record,
                    "hash":t.file_hash,
                    "create_time":t.create_time
                })
            }
        })
        .collect::<Vec<Value>>();
    Ok(JsonData::data(json!({ "data": data,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct ParseRecordDeleteParam {
    pub id: u64,
}

pub async fn parse_record_delete(
    param: &ParseRecordDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .find_by_parse_record_id(&param.id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserBarCodeEdit {
                res_user_id: data.user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .delete_parse_record(auth_data.user_id(), &data, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}
