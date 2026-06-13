use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::app_rbac::{
    AppOpAddParam, AppOpDataParam, AppOpDelParam, AppOpEditParam, app_op_add, app_op_data,
    app_op_del, app_op_edit,
};

#[post("/op/{method}")]
pub async fn op(
    bearer: BearerQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    let data = match path.into_inner().as_str() {
        "add" => app_op_add(&json_param.param::<AppOpAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => app_op_edit(&json_param.param::<AppOpEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => app_op_del(&json_param.param::<AppOpDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => app_op_data(&json_param.param::<AppOpDataParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
