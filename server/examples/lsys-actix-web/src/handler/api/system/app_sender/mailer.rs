use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::system::app_sender::{
    MailerAppSmtpConfigAddParam, MailerSmtpConfigAddParam, MailerSmtpConfigDelParam,
    MailerSmtpConfigEditParam, MailerSmtpConfigListParam, mailer_smtp_config_add,
    mailer_smtp_config_del, mailer_smtp_config_edit, mailer_smtp_config_list,
    mailer_tpl_config_smtp_add,
};
use lsys_web::handler::api::system::app_sender::{
    MailerConfigAddParam, MailerConfigDeleteParam, MailerConfigListParam, MailerMessageBodyParam,
    MailerMessageCancelParam, MailerMessageListParam, MailerMessageLogParam,
    MailerSmtpConfigCheckParam, MailerTplAddParam, MailerTplConfigDelParam,
    MailerTplConfigListParam, MailerTplDelParam, MailerTplEditParam, MailerTplListParam,
    mailer_config_add, mailer_config_del, mailer_config_list, mailer_mapping_data,
    mailer_message_body, mailer_message_cancel, mailer_message_list, mailer_message_log,
    mailer_smtp_config_check, mailer_tpl_body_add, mailer_tpl_body_del, mailer_tpl_body_edit,
    mailer_tpl_body_list, mailer_tpl_config_del, mailer_tpl_config_list,
};

#[post("mailer/{method}")]
pub(crate) async fn mailer(
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
        "mapping" => mailer_mapping_data(&req_query).await,
        "message_logs" => {
            mailer_message_log(&json_param.param::<MailerMessageLogParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_view" => {
            mailer_message_body(&json_param.param::<MailerMessageBodyParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_list" => {
            mailer_message_list(&json_param.param::<MailerMessageListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_cancel" => {
            mailer_message_cancel(&json_param.param::<MailerMessageCancelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }

        "tpl_body_list" => {
            mailer_tpl_body_list(&json_param.param::<MailerTplListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tpl_body_add" => {
            mailer_tpl_body_add(&json_param.param::<MailerTplAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tpl_body_edit" => {
            mailer_tpl_body_edit(&json_param.param::<MailerTplEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tpl_body_del" => {
            mailer_tpl_body_del(&json_param.param::<MailerTplDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }

        "config_add" => {
            mailer_config_add(&json_param.param::<MailerConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "config_del" => {
            mailer_config_del(&json_param.param::<MailerConfigDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "config_list" => {
            mailer_config_list(&json_param.param::<MailerConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tpl_config_list" => {
            mailer_tpl_config_list(&json_param.param::<MailerTplConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "tpl_config_delete" => {
            mailer_tpl_config_del(&json_param.param::<MailerTplConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        //smtpcent
        "smtp_config_list" => {
            mailer_smtp_config_list(&json_param.param::<MailerSmtpConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "smtp_config_add" => {
            mailer_smtp_config_add(&json_param.param::<MailerSmtpConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "smtp_config_check" => {
            mailer_smtp_config_check(
                &json_param.param::<MailerSmtpConfigCheckParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "smtp_config_edit" => {
            mailer_smtp_config_edit(&json_param.param::<MailerSmtpConfigEditParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "smtp_config_del" => {
            mailer_smtp_config_del(&json_param.param::<MailerSmtpConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "smtp_tpl_config_add" => {
            mailer_tpl_config_smtp_add(
                &json_param.param::<MailerAppSmtpConfigAddParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
