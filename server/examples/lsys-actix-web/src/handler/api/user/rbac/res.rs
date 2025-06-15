use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{
    global_res_data_from_tpl_res, user_res_data_from_user_res_type, user_res_type_from_tpl_res,
    UserResDataFromUserResTypeParam,
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
        "tpl_global_res_data" => global_res_data_from_tpl_res(&auth_dao).await,
        "tpl_user_res_type" => user_res_type_from_tpl_res(&auth_dao).await,
        "tpl_user_res_data" => {
            user_res_data_from_user_res_type(
                &json_param.param::<UserResDataFromUserResTypeParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
