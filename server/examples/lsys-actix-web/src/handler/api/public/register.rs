use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult, UserAuthQuery};
use actix_web::post;
use lsys_web::handler::api::auth::{
    user_reg_from_email, user_reg_from_mobile, user_reg_from_name, user_reg_send_code_from_email,
    user_reg_send_code_from_mobile, RegFromEmailParam, RegFromMobileParam, RegFromNameParam,
    RegSendCodeFromEmailParam, RegSendCodeFromMobileParam,
};

#[post("/{type}")]
pub(crate) async fn reg(
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        "name" => user_reg_from_name(&json_param.param::<RegFromNameParam>()?, &auth_dao).await,
        "sms" => user_reg_from_mobile(&json_param.param::<RegFromMobileParam>()?, &auth_dao).await,
        "sms-code" => {
            user_reg_send_code_from_mobile(
                &json_param.param::<RegSendCodeFromMobileParam>()?,
                &auth_dao,
            )
            .await
        }
        "email" => user_reg_from_email(&json_param.param::<RegFromEmailParam>()?, &auth_dao).await,
        "email-code" => {
            user_reg_send_code_from_email(
                &json_param.param::<RegSendCodeFromEmailParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(res.map_err(|e| auth_dao.fluent_error_json_data(&e))?.into())
}
