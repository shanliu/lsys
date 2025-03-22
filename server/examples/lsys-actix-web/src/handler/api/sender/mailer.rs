use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;

use lsys_web::handler::api::user::app_sender::{
    mailer_config_add, mailer_config_del, mailer_config_list, mailer_message_body,
    mailer_message_cancel, mailer_message_list, mailer_message_log, mailer_message_send,
    mailer_smtp_config_list, mailer_tpl_body_add, mailer_tpl_body_del, mailer_tpl_body_edit,
    mailer_tpl_body_list, mailer_tpl_config_del, mailer_tpl_config_list, MailerConfigAddParam,
    MailerConfigDeleteParam, MailerConfigListParam, MailerMessageBodyParam,
    MailerMessageCancelParam, MailerMessageListParam, MailerMessageLogParam,
    MailerMessageSendParam, MailerSmtpConfigListParam, MailerTplAddParam, MailerTplConfigDelParam,
    MailerTplConfigListParam, MailerTplDelParam, MailerTplEditParam, MailerTplListParam,
};
#[post("/mailer/{method}")]
pub(crate) async fn sender_mailer(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "config_add" => {
            mailer_config_add(&json_param.param::<MailerConfigAddParam>()?, &auth_dao).await
        }
        "config_del" => {
            mailer_config_del(&json_param.param::<MailerConfigDeleteParam>()?, &auth_dao).await
        }
        "config_list" => {
            mailer_config_list(&json_param.param::<MailerConfigListParam>()?, &auth_dao).await
        }
        "tpl_config_del" => {
            mailer_tpl_config_del(&json_param.param::<MailerTplConfigDelParam>()?, &auth_dao).await
        }
        "tpl_config_list" => {
            mailer_tpl_config_list(&json_param.param::<MailerTplConfigListParam>()?, &auth_dao)
                .await
        }
        "message_send" => {
            mailer_message_send(&json_param.param::<MailerMessageSendParam>()?, &auth_dao).await
        }
        "message_list" => {
            mailer_message_list(&json_param.param::<MailerMessageListParam>()?, &auth_dao).await
        }
        "message_body" => {
            mailer_message_body(&json_param.param::<MailerMessageBodyParam>()?, &auth_dao).await
        }
        "message_cancel" => {
            mailer_message_cancel(&json_param.param::<MailerMessageCancelParam>()?, &auth_dao).await
        }
        "message_log" => {
            mailer_message_log(&json_param.param::<MailerMessageLogParam>()?, &auth_dao).await
        }
        //SMTP 方式发送邮件相关接口
        "smtp_config_list" => {
            mailer_smtp_config_list(&json_param.param::<MailerSmtpConfigListParam>()?, &auth_dao)
                .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}

#[post("/mailer_tpls/{method}")]
pub(crate) async fn mailer_tpl_body(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "add" => mailer_tpl_body_add(&json_param.param::<MailerTplAddParam>()?, &auth_dao).await,
        "del" => mailer_tpl_body_del(&json_param.param::<MailerTplDelParam>()?, &auth_dao).await,
        "list" => mailer_tpl_body_list(&json_param.param::<MailerTplListParam>()?, &auth_dao).await,
        "edit" => mailer_tpl_body_edit(&json_param.param::<MailerTplEditParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}
