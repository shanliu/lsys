use crate::common::handler::ReqQuery;
use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult};
use actix_web::post;

use lsys_web::handler::api::public::area::{search, CodeParam, GeoParam, SearchParam};

use lsys_web::handler::api::public::area::{code_find, list_data};
use lsys_web::handler::api::public::area::{geo_find, related_find};

#[post("/{type}")]
pub async fn area_data(
    path: actix_web::web::Path<String>,
    req: ReqQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let req_dao = req.inner;
    let res = actix_web::web::block(move || {
        match path.into_inner().as_str() {
            "list" => list_data(&json_param.param::<CodeParam>()?, &req_dao),
            "search" => search(&json_param.param::<SearchParam>()?, &req_dao),
            "related" => related_find(&json_param.param::<CodeParam>()?, &req_dao),
            "find" => code_find(&json_param.param::<CodeParam>()?, &req_dao),
            "geo" => geo_find(&json_param.param::<GeoParam>()?, &req_dao),
            name => handler_not_found!(name),
        }
        .map_err(|e| req_dao.fluent_error_json_response(&e))
    })
    .await?;
    Ok(res?.into())
}
