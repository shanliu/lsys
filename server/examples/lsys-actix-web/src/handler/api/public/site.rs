use crate::common::{
    app::exter_type_data,
    handler::{ResponseJson, ResponseJsonResult, UserAuthQuery},
};
use actix_web::post;

use lsys_web::{
    common::{JsonData, JsonResponse},
    handler::api::public::site::config_data,
};
use serde_json::json;

#[post("/info")]
pub async fn site_info(auth_dao: UserAuthQuery) -> ResponseJsonResult<ResponseJson> {
    let site_setting = config_data(&auth_dao)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "site_tips":site_setting.site_tips,
        "exter_type":exter_type_data()
    })))
    .into())
}
