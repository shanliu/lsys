use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::rbac::{
    DynamicResDataFromUserParam, ResAddParam, ResDelOpParam, ResDelParam, ResEditParam, ResParam,
    ResTypeAddOpParam, ResTypeListParam, ResTypeOpListParam, dynamic_res_data_global_user,
    dynamic_res_type, res_add, res_data, res_del, res_edit, res_type_data, res_type_op_add,
    res_type_op_data, res_type_op_del, static_res_data,
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
        "add" => res_add(&json_param.param::<ResAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "edit" => res_edit(&json_param.param::<ResEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => res_del(&json_param.param::<ResDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => res_data(&json_param.param::<ResParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "static_res_data" => static_res_data(&req_query, &auth_dao, web_dao.as_ref()).await,
        "dynamic_res_type" => dynamic_res_type(&req_query, &auth_dao, web_dao.as_ref()).await,
        "dynamic_res_data_global_user" => {
            //基于user的rbac资源
            dynamic_res_data_global_user(
                &json_param.param::<DynamicResDataFromUserParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "type_data" => res_type_data(&json_param.param::<ResTypeListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "type_op_add" => {
            res_type_op_add(&json_param.param::<ResTypeAddOpParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "type_op_del" => res_type_op_del(&json_param.param::<ResDelOpParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "type_op_data" => {
            res_type_op_data(&json_param.param::<ResTypeOpListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
