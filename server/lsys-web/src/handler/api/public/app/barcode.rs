use crate::common::{JsonError, JsonResult, RequestDao};
use image::ImageFormat;
use lsys_app_barcode::model::BarcodeCreateStatus;
use lsys_core::fluent_message;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct BarCodeShowCodeParam {
    pub contents: String,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub code_id: u64,
}

pub async fn barcode_show(
    param: &BarCodeShowCodeParam,
    req_dao: &RequestDao,
) -> JsonResult<(ImageFormat, Vec<u8>)> {
    let code = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .cache()
        .find_by_create_config_id(&param.code_id)
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(code.app_id)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_BARCODE])
        .await?;
    if !BarcodeCreateStatus::EnablePublic.eq(code.status) {
        return Err(JsonError::Message(fluent_message!(
            "barcode-bad-auth-error"
        )));
    }
    req_dao
        .web_dao
        .app_barcode
        .barcode_show(&param.contents, &code, true)
        .await
}
