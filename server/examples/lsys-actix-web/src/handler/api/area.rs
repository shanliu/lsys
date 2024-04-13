use crate::common::handler::ReqQuery;
use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult};
use actix_web::post;

use lsys_web::handler::api::area::area_search;
use lsys_web::handler::api::area::AreaCodeParam;
use lsys_web::handler::api::area::AreaSearchParam;
use lsys_web::handler::api::area::{area_find, area_list};
use lsys_web::handler::api::area::{area_geo, area_related, AreaGeoParam};

#[post("/{type}")]
pub async fn area_data(
    path: actix_web::web::Path<String>,
    req: ReqQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let req_dao = req.inner;
    let res = actix_web::web::block(move || match path.into_inner().as_str() {
        "list" => area_list(json_param.param::<AreaCodeParam>()?, &req_dao),
        "search" => area_search(json_param.param::<AreaSearchParam>()?, &req_dao),
        "related" => area_related(json_param.param::<AreaCodeParam>()?, &req_dao),
        "find" => area_find(json_param.param::<AreaCodeParam>()?, &req_dao),
        "geo" => area_geo(json_param.param::<AreaGeoParam>()?, &req_dao),
        name => handler_not_found!(name),
    })
    .await?;
    Ok(res?.into())
}
