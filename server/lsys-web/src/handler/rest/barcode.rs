use crate::common::JsonData;
use crate::common::JsonError;
use crate::dao::access::rest::CheckRestApp;
use crate::{common::JsonResponse, common::JsonResult, common::RequestDao};
use lsys_app::model::AppModel;
use lsys_app_barcode::dao::BarcodeParseRecord;
use lsys_app_barcode::dao::ParseParam as BarcodeParseParam;
use lsys_core::fluent_message;
use serde::Deserialize;
use serde_json::json;

use base64::Engine;
use std::path::Path;
#[derive(Debug, Deserialize)]
pub struct ParseParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub try_harder: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub decode_multi: Option<bool>,
    pub barcode_types: Option<Vec<String>>,
    pub other: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub pure_barcode: Option<bool>,
    pub character_set: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_vec_u32")]
    pub allowed_lengths: Option<Vec<u32>>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub assume_code_39_check_digit: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub assume_gs1: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub return_codabar_start_end: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_vec_u32")]
    pub allowed_ean_extensions: Option<Vec<u32>>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub also_inverted: Option<bool>,
}

pub async fn parse_image(
    file_path: impl AsRef<Path>,
    extension: &str,
    param: &ParseParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<serde_json::Value> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp {})
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(app, &[crate::handler::APP_FEATURE_BARCODE])
        .await?;

    match req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .parse(
            app.user_id,
            app.id,
            file_path,
            extension,
            &BarcodeParseParam {
                try_harder: param.try_harder,
                decode_multi: param.decode_multi,
                barcode_types: param
                    .barcode_types
                    .as_ref()
                    .map(|e| e.iter().map(|e| e.as_str()).collect::<Vec<_>>()),
                other: param.other.as_deref(),
                pure_barcode: param.pure_barcode,
                character_set: param.character_set.as_deref(),
                allowed_lengths: param.allowed_lengths.as_deref(),
                assume_code_39_check_digit: param.assume_code_39_check_digit,
                assume_gs1: param.assume_gs1,
                return_codabar_start_end: param.return_codabar_start_end,
                allowed_ean_extensions: param.allowed_ean_extensions.as_deref(),
                also_inverted: param.also_inverted,
            },
            Some(&req_dao.req_env),
        )
        .await
    {
        Ok(tmp) => match tmp {
            BarcodeParseRecord::Succ((t, record)) => Ok(json!({
                "type":t.barcode_type,
                "text":record.text,
                "position":record.position,
                "hash":t.file_hash,
            })),
            BarcodeParseRecord::Fail(t) => Err(JsonError::Message(fluent_message!(
                "barcode-parse-error",
                {"record":t.record}
            ))),
        },
        Err(err) => Err(err.into()),
    }
}

#[derive(Debug, Deserialize)]
pub struct CodeParam {
    pub contents: String,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub code_id: u64,
}

pub async fn barcode_base64(
    param: &CodeParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(app, &[crate::handler::APP_FEATURE_BARCODE])
        .await?;

    let code = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .cache()
        .find_by_create_config_id(&param.code_id)
        .await?;

    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_show(&param.contents, &code, true)
        .await?;
    let base64 = base64::engine::general_purpose::STANDARD.encode(data.1);
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": base64,
        "type":data.0.to_mime_type()
    }))))
}
