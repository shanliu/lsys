use crate::common::handler::{ResponseJson, ResponseJsonResult, UserAuthQuery};
use actix_web::get;

use lsys_web::handler::api::public::site::config_info;

#[get("/info")]
pub async fn site_info(auth_dao: UserAuthQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(config_info(&auth_dao)
        .await
        .map_err(|e| auth_dao.fluent_error_json_data(&e))?
        .into())
}
