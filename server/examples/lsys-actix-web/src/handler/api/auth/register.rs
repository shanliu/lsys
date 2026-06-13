use crate::common::handler::{JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::auth::{
    RegFromEmailParam, RegFromMobileParam, RegFromNameParam, RegSendCodeFromEmailParam,
    RegSendCodeFromMobileParam, user_reg_from_email, user_reg_from_mobile, user_reg_from_name,
    user_reg_send_code_from_email, user_reg_send_code_from_mobile,
};

#[post("register/{type}")]
pub(crate) async fn register(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    req_dao: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        "name" => user_reg_from_name(&json_param.param::<RegFromNameParam>()?, &req_dao, web_dao.as_ref()).await,
        "sms" => user_reg_from_mobile(&json_param.param::<RegFromMobileParam>()?, &req_dao, web_dao.as_ref()).await,
        "sms-code" => {
            user_reg_send_code_from_mobile(
                &json_param.param::<RegSendCodeFromMobileParam>()?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "email" => user_reg_from_email(&json_param.param::<RegFromEmailParam>()?, &req_dao, web_dao.as_ref()).await,
        "email-code" => {
            user_reg_send_code_from_email(
                &json_param.param::<RegSendCodeFromEmailParam>()?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}
