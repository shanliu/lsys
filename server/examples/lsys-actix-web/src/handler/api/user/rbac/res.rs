use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::rbac::{
    UserResDataFromUserResTypeParam, dynamic_res_type, dynamic_res_type_from_test, static_res_data,
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
        "static_res_data" => static_res_data(&req_query, &auth_dao, web_dao.as_ref()).await,
        "dynamic_res_type" => dynamic_res_type(&req_query, &auth_dao, web_dao.as_ref()).await,
        "dynamic_res_data_test" => {
            dynamic_res_type_from_test(
                &json_param.param::<UserResDataFromUserResTypeParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_query.fluent_error_json_response(&e))?
        .into())
}
