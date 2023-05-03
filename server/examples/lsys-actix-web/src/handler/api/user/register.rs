use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult, UserAuthQuery};
use actix_web::post;
use lsys_web::handler::api::user::{
    user_reg_from_email, user_reg_from_mobile, user_reg_from_name, user_reg_send_code_from_email,
    user_reg_send_code_from_mobile, RegFromEmailParam, RegFromMobileParam, RegFromNameParam,
    RegSendCodeFromEmailParam, RegSendCodeFromMobileParam,
};

#[post("/signup/{type}")]
pub(crate) async fn reg<'t>(
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.0.to_string().as_str() {
        "name" => user_reg_from_name(json_param.param::<RegFromNameParam>()?, &auth_dao).await,
        "sms" => user_reg_from_mobile(json_param.param::<RegFromMobileParam>()?, &auth_dao).await,
        "sms-code" => {
            user_reg_send_code_from_mobile(
                json_param.param::<RegSendCodeFromMobileParam>()?,
                &auth_dao,
            )
            .await
        }
        "email" => user_reg_from_email(json_param.param::<RegFromEmailParam>()?, &auth_dao).await,
        "email-code" => {
            user_reg_send_code_from_email(
                json_param.param::<RegSendCodeFromEmailParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}
