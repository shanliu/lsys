use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult, UserAuthQuery};
use actix_web::post;
use lsys_web::handler::api::auth::{
    user_reset_password_from_email, user_reset_password_from_mobile,
    user_reset_password_send_code_from_email, user_reset_password_send_code_from_mobile,
    ResetPasswordFromEmailParam, ResetPasswordFromMobileParam, ResetPasswordSendCodeFromEmailParam,
    ResetPasswordSendCodeFromMobileParam,
};

#[post("/{method}")]
pub(crate) async fn password_reset(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match path.into_inner().as_str() {
        "email" => {
            user_reset_password_from_email(
                &json_param.param::<ResetPasswordFromEmailParam>()?,
                &auth_dao,
            )
            .await
        }
        "mobile" => {
            user_reset_password_from_mobile(
                &json_param.param::<ResetPasswordFromMobileParam>()?,
                &auth_dao,
            )
            .await
        }
        "email_code" => {
            user_reset_password_send_code_from_email(
                &json_param.param::<ResetPasswordSendCodeFromEmailParam>()?,
                &auth_dao,
            )
            .await
        }
        "mobile_code" => {
            user_reset_password_send_code_from_mobile(
                &json_param.param::<ResetPasswordSendCodeFromMobileParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}
