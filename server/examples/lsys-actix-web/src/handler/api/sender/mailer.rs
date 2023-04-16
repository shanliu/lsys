use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::sender::{
    mailer_app_smtp_config_add, mailer_app_smtp_config_del, mailer_app_smtp_config_list,
    mailer_config_add, mailer_config_del, mailer_config_list, mailer_message_body,
    mailer_message_cancel, mailer_message_list, mailer_message_log, mailer_smtp_config_add,
    mailer_smtp_config_check, mailer_smtp_config_del, mailer_smtp_config_edit,
    mailer_smtp_config_list, MailerAppSmtpConfigAddParam, MailerAppSmtpConfigDelParam,
    MailerAppSmtpConfigListParam, MailerConfigAddParam, MailerConfigDeleteParam,
    MailerConfigListParam, MailerMessageBodyParam, MailerMessageCancelParam,
    MailerMessageListParam, MailerMessageLogParam, MailerSmtpConfigAddParam,
    MailerSmtpConfigCheckParam, MailerSmtpConfigDelParam, MailerSmtpConfigEditParam,
    MailerSmtpConfigListParam,
};
#[post("/mailer/{method}")]
pub(crate) async fn sender_mailer<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "config_add" => mailer_config_add(rest.param::<MailerConfigAddParam>()?, &auth_dao).await,
        "config_del" => {
            mailer_config_del(rest.param::<MailerConfigDeleteParam>()?, &auth_dao).await
        }
        "config_list" => {
            mailer_config_list(rest.param::<MailerConfigListParam>()?, &auth_dao).await
        }
        "message_list" => {
            mailer_message_list(rest.param::<MailerMessageListParam>()?, &auth_dao).await
        }
        "message_body" => {
            mailer_message_body(rest.param::<MailerMessageBodyParam>()?, &auth_dao).await
        }
        "message_cancel" => {
            mailer_message_cancel(rest.param::<MailerMessageCancelParam>()?, &auth_dao).await
        }
        "message_log" => {
            mailer_message_log(rest.param::<MailerMessageLogParam>()?, &auth_dao).await
        }
        "smtp_config_list" => {
            mailer_smtp_config_list(rest.param::<MailerSmtpConfigListParam>()?, &auth_dao).await
        }
        "smtp_config_add" => {
            mailer_smtp_config_add(rest.param::<MailerSmtpConfigAddParam>()?, &auth_dao).await
        }
        "smtp_config_check" => {
            mailer_smtp_config_check(rest.param::<MailerSmtpConfigCheckParam>()?, &auth_dao).await
        }
        "smtp_config_edit" => {
            mailer_smtp_config_edit(rest.param::<MailerSmtpConfigEditParam>()?, &auth_dao).await
        }
        "smtp_config_del" => {
            mailer_smtp_config_del(rest.param::<MailerSmtpConfigDelParam>()?, &auth_dao).await
        }
        "smtp_app_config_add" => {
            mailer_app_smtp_config_add(rest.param::<MailerAppSmtpConfigAddParam>()?, &auth_dao)
                .await
        }
        "smtp_app_config_del" => {
            mailer_app_smtp_config_del(rest.param::<MailerAppSmtpConfigDelParam>()?, &auth_dao)
                .await
        }
        "smtp_app_config_list" => {
            mailer_app_smtp_config_list(rest.param::<MailerAppSmtpConfigListParam>()?, &auth_dao)
                .await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
