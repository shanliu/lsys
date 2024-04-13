use crate::handler::common::barcode::{barcode_show, BarCodeShowParam};
use crate::{dao::RequestDao, JsonData, JsonResult};
use lsys_app_barcode::dao::{BarCodeError, BarcodeParseRecord};
use lsys_app_barcode::dao::ParseParam;

use serde::Deserialize;
use serde_json::json;

use std::path::Path;
#[derive(Debug, Deserialize)]
pub struct BarCodeParseParam {
    pub try_harder: Option<bool>,
    pub decode_multi: Option<bool>,
    pub barcode_types: Option<Vec<String>>,
    pub other: Option<String>,
    pub pure_barcode: Option<bool>,
    pub character_set: Option<String>,
    pub allowed_lengths: Option<Vec<u32>>,
    pub assume_code_39_check_digit: Option<bool>,
    pub assume_gs1: Option<bool>,
    pub return_codabar_start_end: Option<bool>,
    pub allowed_ean_extensions: Option<Vec<u32>>,
    pub also_inverted: Option<bool>,
}

pub async fn barcode_parse(
    file: impl AsRef<Path>,
    extension: &str,
    user_id: &u64,
    app_id: &u64,
    param: &BarCodeParseParam,
    req_dao: &RequestDao,
) -> Result<BarcodeParseRecord, BarCodeError> {
    let parse_param = ParseParam {
        try_harder: param.try_harder,
        decode_multi: param.decode_multi,
        barcode_types: param.barcode_types.to_owned(),
        other: param.other.to_owned(),
        pure_barcode: param.pure_barcode,
        character_set: param.character_set.to_owned(),
        allowed_lengths: param.allowed_lengths.to_owned(),
        assume_code_39_check_digit: param.assume_code_39_check_digit,
        assume_gs1: param.assume_gs1,
        return_codabar_start_end: param.return_codabar_start_end,
        allowed_ean_extensions: param.allowed_ean_extensions.to_owned(),
        also_inverted: param.also_inverted,
    };
    req_dao
        .web_dao
        .barcode
        .parse(
            user_id,
            app_id,
            file,
            extension,
            &parse_param,
            Some(&req_dao.req_env),
        )
        .await
}


use base64::Engine;
pub async fn barcode_show_base64(
    param: &BarCodeShowParam,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let data = barcode_show(param, req_dao).await?;
    let base64 = base64::engine::general_purpose::STANDARD.encode(data.1);
    Ok(JsonData::data(
        json!({ "data": base64,"type":data.0.to_mime_type() }),
    ))
}
