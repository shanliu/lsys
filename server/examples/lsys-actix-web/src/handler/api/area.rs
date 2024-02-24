use crate::common::handler::ReqQuery;
use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult};
use actix_web::post;

use lsys_web::handler::api::utils::area_detail;
use lsys_web::handler::api::utils::area_list;
use lsys_web::handler::api::utils::area_search;
use lsys_web::handler::api::utils::AreaCodeParam;
use lsys_web::handler::api::utils::AreaSearchParam;

#[post("/{type}")]
pub async fn area_data(
    path: actix_web::web::Path<String>,
    req: ReqQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let req_dao = req.inner;
    let res = actix_web::web::block(move || match path.into_inner().as_str() {
        "list" => area_list(json_param.param::<AreaCodeParam>()?, &req_dao),
        "detail" => area_detail(json_param.param::<AreaCodeParam>()?, &req_dao),
        "search" => area_search(json_param.param::<AreaSearchParam>()?, &req_dao),
        name => handler_not_found!(name),
    })
    .await?;
    Ok(res?.into())
}
