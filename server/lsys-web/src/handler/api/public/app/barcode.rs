use crate::{
    common::{JsonResult, RequestDao},
    dao::APP_FEATURE_BARCODE,
};
use image::ImageFormat;
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct BarCodeShowParam {
    pub contents: String,
    pub code_id: u64,
}

pub async fn app_barcode_show(
    param: &BarCodeShowParam,
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
        .find_by_id(&code.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[APP_FEATURE_BARCODE])
        .await?;
    req_dao
        .web_dao
        .app_barcode
        .barcode_show(&param.contents, &code, true)
        .await
}
