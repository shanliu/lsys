use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::app_rbac::{
    AppResAddParam, AppResDelOpParam, AppResDelParam, AppResEditParam, AppResParam,
    AppResTypeAddOpParam, AppResTypeListParam, AppResTypeOpListParam, app_res_add, app_res_data,
    app_res_del, app_res_edit, app_res_type_data, app_res_type_op_add, app_res_type_op_data,
    app_res_type_op_del,
};

#[post("/res/{method}")]
pub async fn res(
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
        "add" => app_res_add(&json_param.param::<AppResAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => app_res_edit(&json_param.param::<AppResEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => app_res_del(&json_param.param::<AppResDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => app_res_data(&json_param.param::<AppResParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "type_data" => {
            app_res_type_data(&json_param.param::<AppResTypeListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "type_op_add" => {
            app_res_type_op_add(&json_param.param::<AppResTypeAddOpParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "type_op_del" => {
            app_res_type_op_del(&json_param.param::<AppResDelOpParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "type_op_data" => {
            app_res_type_op_data(&json_param.param::<AppResTypeOpListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
