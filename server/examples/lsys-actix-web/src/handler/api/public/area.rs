use crate::common::handler::ReqQuery;
use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::public::area::{CodeParam, GeoParam, SearchParam, search};

use lsys_web::handler::api::public::area::{code_find, list_data};
use lsys_web::handler::api::public::area::{geo_find, related_find};

#[post("/{type}")]
pub async fn area_data(
    path: actix_web::web::Path<String>,
    req: ReqQuery,
    json_param: JsonQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let web_dao_arc = web_dao.into_inner();
    let res = actix_web::web::block(move || {
        match path.into_inner().as_str() {
            "list" => list_data(&json_param.param::<CodeParam>()?, &web_dao_arc),
            "search" => search(&json_param.param::<SearchParam>()?, &web_dao_arc),
            "related" => related_find(&json_param.param::<CodeParam>()?, &web_dao_arc),
            "find" => code_find(&json_param.param::<CodeParam>()?, &web_dao_arc),
            "geo" => geo_find(&json_param.param::<GeoParam>()?, &web_dao_arc),
            name => handler_not_found!(name),
        }
        .map_err(|e| req.fluent_error_json_response(&e))
    })
    .await?;
    Ok(res?.into())
}
