use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::rbac::{
    dynamic_res_type, dynamic_res_type_from_test, static_res_data, UserResDataFromUserResTypeParam,
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
        "static_res_data" => static_res_data(&auth_dao).await,
        "dynamic_res_type" => dynamic_res_type(&auth_dao).await,
        "dynamic_res_data_test" => {
            dynamic_res_type_from_test(
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
