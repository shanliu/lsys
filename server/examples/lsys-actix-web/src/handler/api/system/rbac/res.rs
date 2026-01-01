use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::system::rbac::{
    dynamic_res_data_global_user, dynamic_res_type, res_add, res_data, res_del, res_edit,
    res_type_data, res_type_op_add, res_type_op_data, res_type_op_del, static_res_data,
    DynamicResDataFromUserParam, ResAddParam, ResDelOpParam, ResDelParam, ResEditParam, ResParam,
    ResTypeAddOpParam, ResTypeListParam, ResTypeOpListParam,
};

#[post("/res/{method}")]
pub async fn res(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    let data = match path.into_inner().as_str() {
        "add" => res_add(&json_param.param::<ResAddParam>()?, &auth_dao).await,
        "edit" => res_edit(&json_param.param::<ResEditParam>()?, &auth_dao).await,
        "delete" => res_del(&json_param.param::<ResDelParam>()?, &auth_dao).await,
        "list" => res_data(&json_param.param::<ResParam>()?, &auth_dao).await,
        "static_res_data" => static_res_data(&auth_dao).await,
        "dynamic_res_type" => dynamic_res_type(&auth_dao).await,
        "dynamic_res_data_global_user" => {
            //基于user的rbac资源
            dynamic_res_data_global_user(
                &json_param.param::<DynamicResDataFromUserParam>()?,
                &auth_dao,
            )
            .await
        }
        "type_data" => res_type_data(&json_param.param::<ResTypeListParam>()?, &auth_dao).await,
        "type_op_add" => {
            res_type_op_add(&json_param.param::<ResTypeAddOpParam>()?, &auth_dao).await
        }
        "type_op_del" => res_type_op_del(&json_param.param::<ResDelOpParam>()?, &auth_dao).await,
        "type_op_data" => {
            res_type_op_data(&json_param.param::<ResTypeOpListParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
