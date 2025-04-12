use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{
    app_res_add, app_res_data, app_res_del, app_res_edit, app_res_type_data, app_res_type_op_add,
    app_res_type_op_data, app_res_type_op_del, AppResAddParam, AppResDelOpParam, AppResDelParam,
    AppResEditParam, AppResParam, AppResTypeAddOpParam, AppResTypeListParam, AppResTypeOpListParam,
};

#[post("/res/{method}")]
pub async fn res(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let data = match path.into_inner().as_str() {
        "add" => app_res_add(&json_param.param::<AppResAddParam>()?, &auth_dao).await,
        "edit" => app_res_edit(&json_param.param::<AppResEditParam>()?, &auth_dao).await,
        "delete" => app_res_del(&json_param.param::<AppResDelParam>()?, &auth_dao).await,
        "list" => app_res_data(&json_param.param::<AppResParam>()?, &auth_dao).await,
        "type_data" => {
            app_res_type_data(&json_param.param::<AppResTypeListParam>()?, &auth_dao).await
        }
        "type_op_add" => {
            app_res_type_op_add(&json_param.param::<AppResTypeAddOpParam>()?, &auth_dao).await
        }
        "type_op_del" => {
            app_res_type_op_del(&json_param.param::<AppResDelOpParam>()?, &auth_dao).await
        }
        "type_op_data" => {
            app_res_type_op_data(&json_param.param::<AppResTypeOpListParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
