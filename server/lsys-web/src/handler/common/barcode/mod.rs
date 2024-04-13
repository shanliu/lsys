use crate::dao::RequestAuthDao;
use crate::handler::access::{AccessBarCodeEdit, AccessBarCodeView};
use crate::PageParam;
use crate::{dao::RequestDao, JsonData, JsonResult};
use image::ImageFormat;
use lsys_app_barcode::dao::BarcodeParseRecord;
use lsys_app_barcode::model::BarcodeCreateStatus;
use lsys_core::fluent_message;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
use std::io::Cursor;

#[derive(Debug, Deserialize)]
pub struct BarCodeShowParam {
    pub contents: String,
    pub code_id: u64,
}

pub async fn barcode_show(
    param: &BarCodeShowParam,
    req_dao: &RequestDao,
) -> JsonResult<(ImageFormat, Vec<u8>)> {
    let image_buffer = req_dao
        .web_dao
        .barcode
        .create(&param.code_id, &param.contents)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if BarcodeCreateStatus::EnablePublic.eq(image_buffer.1.status) {
        return Err(req_dao.fluent_json_data(fluent_message!("barcode-bad-auth-error")));
    }
    let mut png_data: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(&mut png_data);
    let image_format = match ImageFormat::from_extension(&image_buffer.1.image_format) {
        Some(t) => t,
        None => {
            return Err(
                req_dao.fluent_json_data(fluent_message!("barcode-bad-format-error",{
                    "foramt":image_buffer.1.image_format
                })),
            )
        }
    };
    image_buffer
        .0
        .write_to(&mut cursor, image_format)
        .map_err(|e| {
            req_dao.fluent_json_data(fluent_message!("barcode-bad-image-error",{
                "err":e
            }))
        })?;
    Ok((image_format, png_data))
}

#[derive(Debug, Deserialize)]
pub struct BarCodeCreateConfigListParam {
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn barcode_create_config_list<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: BarCodeCreateConfigListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessBarCodeView {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let data = req_dao
        .web_dao
        .barcode
        .list_create_config(
            &req_auth.user_data().user_id,
            &param.app_id,
            &param.barcode_type,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .barcode
                .count_create_config(
                    &req_auth.user_data().user_id,
                    &param.app_id,
                    &param.barcode_type,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": data,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct BarCodeParseRecordListParam {
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn barcode_parse_record_list<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: BarCodeParseRecordListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessBarCodeView {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let data = req_dao
        .web_dao
        .barcode
        .list_parse_record(
            &req_auth.user_data().user_id,
            &param.app_id,
            &param.barcode_type,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await 
        .map_err(|e| req_dao.fluent_json_data(e))
        ?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .barcode
                .count_parse_record(
                    &req_auth.user_data().user_id,
                    &param.app_id,
                    &param.barcode_type,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };
    let data=data.into_iter().map(|tmp|{
        match tmp{
            BarcodeParseRecord::Succ(t) => {
                json!({
                    "id":t.0.id,
                    "app_id":t.0.app_id,
                    "bar_type":t.0.barcode_type,
                    "status":1,
                    "text":t.1.text,
                    "error":"",
                    "hash":t.0.file_hash
                })
            },
            BarcodeParseRecord::Fail(t) => {
                json!({
                    "id":t.id,
                    "app_id":t.app_id,
                    "bar_type":t.barcode_type,
                    "text":"",
                    "status":0,
                    "error":t.record,
                    "hash":t.file_hash
                })
            },
        }
    }).collect::<Vec<Value>>();
    Ok(JsonData::data(json!({ "data": data,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct BarCodeParseRecordDeleteParam {
    pub id: u64,
}

pub async fn barcode_parse_record_delete<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: BarCodeParseRecordDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessBarCodeEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let data = req_dao
        .web_dao
        .barcode
        .find_by_parse_record_id(&param.id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .barcode
        .delete_parse_record(&req_auth.user_data().user_id, &data, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct BarCodeCreateConfigDeleteParam {
    pub id: u64,
}

pub async fn barcode_create_config_delete<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: BarCodeCreateConfigDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessBarCodeEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let data = req_dao
        .web_dao
        .barcode
        .find_by_create_config_id(&param.id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .barcode
        .delete_create_config(&req_auth.user_data().user_id, &data, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
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

pub async fn barcode_create_config_add<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: BarCodeCreateConfigAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessBarCodeEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let status = BarcodeCreateStatus::try_from(param.status)
        .map_err(|e| req_dao.fluent_json_data(fluent_message!("barcode-add-status-error", e)))?;

   let id= req_dao
        .web_dao
        .barcode
        .add_create_config(
            &req_auth.user_data().user_id,
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
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

pub async fn barcode_create_config_edit<
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: BarCodeCreateConfigEditParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessBarCodeEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let data = req_dao
        .web_dao
        .barcode
        .find_by_create_config_id(&param.id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;

    let status = BarcodeCreateStatus::try_from(param.status)
        .map_err(|e| req_dao.fluent_json_data(fluent_message!("barcode-add-status-error", e)))?;

    req_dao
        .web_dao
        .barcode
        .edit_create_config(
            &data,
            &req_auth.user_data().user_id,
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}
