use crate::common::handler::{JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::auth::{
    ResetPasswordFromEmailParam, ResetPasswordFromMobileParam, ResetPasswordSendCodeFromEmailParam,
    ResetPasswordSendCodeFromMobileParam, user_reset_password_from_email,
    user_reset_password_from_mobile, user_reset_password_send_code_from_email,
    user_reset_password_send_code_from_mobile,
};

#[post("password/{method}")]
pub(crate) async fn password(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    req_dao: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match path.into_inner().as_str() {
        "email" => {
            user_reset_password_from_email(
                &json_param.param::<ResetPasswordFromEmailParam>()?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "mobile" => {
            user_reset_password_from_mobile(
                &json_param.param::<ResetPasswordFromMobileParam>()?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "email_code" => {
            user_reset_password_send_code_from_email(
                &json_param.param::<ResetPasswordSendCodeFromEmailParam>()?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "mobile_code" => {
            user_reset_password_send_code_from_mobile(
                &json_param.param::<ResetPasswordSendCodeFromMobileParam>()?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_dao.fluent_error_json_response(&e))?
    .into())
}
