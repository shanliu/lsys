use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::{
    user_reset_password_from_email, user_reset_password_from_mobile,
    user_reset_password_send_code_from_email, user_reset_password_send_code_from_mobile,
    user_set_password, ResetPasswordFromEmailParam, ResetPasswordFromMobileParam,
    ResetPasswordSendCodeFromEmailParam, ResetPasswordSendCodeFromMobileParam, SetPasswordParam,
};

#[post("password_reset/{method}")]
pub(crate) async fn password_reset<'t>(
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match path.0.to_string().as_str() {
        "email" => {
            user_reset_password_from_email(&rest.param::<ResetPasswordFromEmailParam>()?, &auth_dao)
                .await
        }
        "mobile" => {
            user_reset_password_from_mobile(
                &rest.param::<ResetPasswordFromMobileParam>()?,
                &auth_dao,
            )
            .await
        }
        "email_code" => {
            user_reset_password_send_code_from_email(
                rest.param::<ResetPasswordSendCodeFromEmailParam>()?,
                &auth_dao,
            )
            .await
        }
        "mobile_code" => {
            user_reset_password_send_code_from_mobile(
                rest.param::<ResetPasswordSendCodeFromMobileParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    }?
    .into())
}

#[post("password/{method}")]
pub(crate) async fn password<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "set" => user_set_password(&rest.param::<SetPasswordParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
