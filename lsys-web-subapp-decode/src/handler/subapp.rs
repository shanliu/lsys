use lsys_app::model::AppsModel;
use lsys_web::{dao::WebDao, JsonData, JsonResult};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ImageDataParam {
    pub base64_image: Vec<u8>,
}
pub async fn decode_image(
    app_dao: &WebDao,
    app: &AppsModel,
    _param: ImageDataParam,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            app.user_id,
            &[app_dao.app.app_relation_key(app).await],
            &res_data!(AppDecode(app.id)),
        )
        .await?;
    Ok(JsonData::message("set name succ").set_data(json!({ "pass": 1 })))
}
