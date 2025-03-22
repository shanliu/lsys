use crate::common::{JsonResult, RequestDao};
use image::ImageFormat;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct BarCodeShowCodeParam {
    pub contents: String,
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

    req_dao
        .web_dao
        .app_barcode
        .app_feature_check_from_app_id(code.app_id)
        .await?;

    req_dao
        .web_dao
        .app_barcode
        .barcode_show(&param.contents, &code, true)
        .await
}
