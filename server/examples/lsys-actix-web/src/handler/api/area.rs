use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::utils::area_detail;
use lsys_web::handler::api::utils::area_list;
use lsys_web::handler::api::utils::area_search;
use lsys_web::handler::api::utils::AreaCodeParam;
use lsys_web::handler::api::utils::AreaSearchParam;

#[post("/{type}")]
pub async fn area_data(
    path: actix_web::web::Path<(String,)>,
    web_dao: Data<WebDao>,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = actix_web::web::block(move || match path.0.to_string().as_str() {
        "list" => area_list(json_param.param::<AreaCodeParam>()?, &web_dao),
        "detail" => area_detail(json_param.param::<AreaCodeParam>()?, &web_dao),
        "search" => area_search(json_param.param::<AreaSearchParam>()?, &web_dao),
        name => handler_not_found!(name),
    })
    .await?;
    Ok(res?.into())
}
