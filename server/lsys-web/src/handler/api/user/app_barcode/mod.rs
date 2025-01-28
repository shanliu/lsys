use lsys_access::dao::AccessSession;
use lsys_app_barcode::dao::BarcodeParseRecord;
use lsys_app_barcode::model::BarcodeCreateModel;
use lsys_app_barcode::model::BarcodeCreateStatus;
use lsys_core::fluent_message;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;

use crate::common::JsonData;
use crate::common::JsonError;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::user::CheckBarCodeEdit;
use crate::dao::access::api::user::CheckBarCodeView;

#[derive(Debug, Deserialize)]
pub struct BarCodeCreateConfigListParam {
    pub id: Option<u64>,
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

#[derive(Debug, Deserialize)]
pub struct BarCodeParseRecordListParam {
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn barcode_parse_record_list(
    param: &BarCodeParseRecordListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckBarCodeView {}, None)
        .await?;
    // req_dao
    //     .web_dao
    //     .rbac
    //     .check(&req_dao.access_env().await?, &CheckApp {}, None)
    //     .await?;

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
pub struct BarCodeParseRecordDeleteParam {
    pub id: u64,
}

pub async fn barcode_parse_record_delete(
    param: &BarCodeParseRecordDeleteParam,
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
            &req_dao.access_env().await?,
            &CheckBarCodeEdit {
                app_id: data.app_id,
            },
            None,
        )
        .await?;

    // req_dao
    //     .web_dao
    //     .rbac
    //     .check(
    //         &req_dao.access_env().await?,
    //         &CheckBarCodeEdit {
    //             user_id: auth_data.user_id(),
    //             app_id: data.app_id,
    //         },
    //         None,
    //     )
    //     .await?;
    req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .delete_parse_record(auth_data.user_id(), &data, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct BarCodeCreateConfigDeleteParam {
    pub id: u64,
}

pub async fn barcode_create_config_delete(
    param: &BarCodeCreateConfigDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .find_by_create_config_id(&param.id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckBarCodeEdit {
                app_id: data.app_id,
            },
            None,
        )
        .await?;

    // req_dao
    //     .web_dao
    //     .rbac
    //     .check(
    //         &req_dao.access_env().await?,
    //         &CheckBarCodeEdit {
    //             user_id: auth_data.user_id(),
    //             app_id: data.app_id,
    //         },
    //         None,
    //     )
    //     .await?;
    req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .delete_create_config(&auth_data.user_id(), &data, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct BarCodeCreateConfigAddParam {
    pub app_id: u64,
    pub barcode_type: String,
    pub status: i8,
    pub image_format: String,
    pub image_width: i32,
    pub image_height: i32,
    pub margin: i32,
    pub image_color: String,
    pub image_background: String,
}

pub async fn barcode_create_config_add(
    param: &BarCodeCreateConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckBarCodeEdit {
                app_id: param.app_id,
            },
            None,
        )
        .await?;

    // req_dao
    //     .web_dao
    //     .rbac
    //     .check(
    //         &req_dao.access_env().await?,
    //         &CheckBarCodeEdit {
    //             user_id: auth_data.user_id(),
    //             app_id: param.app_id,
    //         },
    //         None,
    //     )
    //     .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let status = BarcodeCreateStatus::try_from(param.status)
        .map_err(|e| JsonError::Message(fluent_message!("barcode-add-status-error", e)))?;

    let id = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .add_create_config(
            &auth_data.user_id(),
            &param.app_id,
            &status,
            &param.barcode_type,
            &param.image_format,
            &param.image_width,
            &param.image_height,
            &param.margin,
            &param.image_color,
            &param.image_background,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({
        "id":id,
    })))
}

#[derive(Debug, Deserialize)]
pub struct BarCodeCreateConfigEditParam {
    pub id: u64,
    pub barcode_type: String,
    pub status: i8,
    pub image_format: String,
    pub image_width: i32,
    pub image_height: i32,
    pub margin: i32,
    pub image_color: String,
    pub image_background: String,
}

pub async fn barcode_create_config_edit(
    param: &BarCodeCreateConfigEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckBarCodeEdit { app_id: param.id },
            None,
        )
        .await?;

    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .find_by_create_config_id(&param.id)
        .await?;

    // req_dao
    //     .web_dao
    //     .rbac
    //     .check(
    //         &req_dao.access_env().await?,
    //         &CheckBarCodeEdit {
    //             user_id: auth_data.user_id(),
    //             app_id: data.app_id,
    //         },
    //         None,
    //     )
    //     .await?;

    let status = BarcodeCreateStatus::try_from(param.status)
        .map_err(|e| JsonError::Message(fluent_message!("barcode-add-status-error", e)))?;

    req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .edit_create_config(
            &data,
            &auth_data.user_id(),
            &status,
            &param.barcode_type,
            &param.image_format,
            &param.image_width,
            &param.image_height,
            &param.margin,
            &param.image_color,
            &param.image_background,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

pub async fn barcode_create_config_list(
    param: &BarCodeCreateConfigListParam,
    req_dao: &UserAuthQueryDao,
    url_callback: impl Fn(&BarcodeCreateModel) -> String,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckBarCodeView {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .list_create_config(
            auth_data.user_id(),
            param.id,
            param.app_id,
            param.barcode_type.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?
        .into_iter()
        .map(|e| {
            let url = url_callback(&e);
            json!({
                "id":e.id,
                "barcode_type":e.barcode_type,
                "app_id":e.app_id,
                "change_time":e.change_time,
                "image_background":e.image_background,
                "image_color":e.image_color,
                "image_format":e.image_format,
                "image_height":e.image_height,
                "image_width":e.image_width,
                "margin":e.margin,
                "status":e.status,
                "url":url,
            })
        })
        .collect::<Vec<_>>();
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_barcode
                .barcode_dao
                .count_create_config(
                    auth_data.user_id(),
                    param.id,
                    param.app_id,
                    param.barcode_type.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": data,"total":count })))
}
