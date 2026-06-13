use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::rbac::{
    OpAddParam, OpDataParam, OpDelParam, OpEditParam, op_add, op_data, op_del, op_edit,
};

#[post("/op")]
pub async fn op(
    rest: RestQuery,
    req_dao: ReqQuery,
    web_dao: web::Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let data = match rest.rfc.method.as_deref().unwrap_or_default() {
        "add" => op_add(&rest.param::<OpAddParam>()?, &rest.get_app().await?, &req_dao, &web_dao).await,
        "edit" => op_edit(&rest.param::<OpEditParam>()?, &rest.get_app().await?, &req_dao, &web_dao).await,
        "delete" => op_del(&rest.param::<OpDelParam>()?, &rest.get_app().await?, &req_dao, &web_dao).await,
        "list" => op_data(&rest.param::<OpDataParam>()?, &rest.get_app().await?, &req_dao, &web_dao).await,
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}
