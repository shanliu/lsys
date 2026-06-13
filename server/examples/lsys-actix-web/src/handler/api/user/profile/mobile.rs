use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::account::{
    MobileAddParam, MobileConfirmParam, MobileDeleteParam, MobileListDataParam,
    MobileSendCodeParam, mobile_add, mobile_confirm, mobile_delete, mobile_list_data,
    mobile_send_code,
};

#[post("/mobile/{method}")]
pub(crate) async fn mobile(
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
    Ok(match path.into_inner().as_str() {
        "add" => mobile_add(&json_param.param::<MobileAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "send_code" => {
            mobile_send_code(&json_param.param::<MobileSendCodeParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "delete" => mobile_delete(&json_param.param::<MobileDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "confirm" => mobile_confirm(&json_param.param::<MobileConfirmParam>()?, &req_query, web_dao.as_ref()).await,
        "list_data" => {
            mobile_list_data(&json_param.param::<MobileListDataParam>()?, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
