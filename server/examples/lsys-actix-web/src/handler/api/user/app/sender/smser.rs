use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::{post, HttpRequest};
use lsys_web::handler::api::user::app_sender::{
    smser_ali_app_config_add, smser_ali_config_list, smser_cloopen_app_config_add,
    smser_cloopen_config_list, smser_config_add, smser_config_del, smser_config_list,
    smser_hw_app_config_add, smser_hw_config_list, smser_jd_app_config_add, smser_jd_config_list,
    smser_message_body, smser_message_cancel, smser_message_list, smser_message_log,
    smser_message_send, smser_netease_app_config_add, smser_netease_config_list,
    smser_notify_get_config, smser_notify_set_config, smser_ten_app_config_add,
    smser_ten_config_list, smser_tpl_config_del, smser_tpl_config_list, SmserAliConfigListParam,
    SmserAppAliConfigAddParam, SmserAppCloopenConfigAddParam, SmserAppHwConfigAddParam,
    SmserAppJDConfigAddParam, SmserAppNetEaseConfigAddParam, SmserAppTenConfigAddParam,
    SmserCloOpenConfigListParam, SmserConfigAddParam, SmserConfigDeleteParam, SmserConfigListParam,
    SmserHwConfigListParam, SmserJDConfigListParam, SmserMessageBodyParam, SmserMessageCancelParam,
    SmserMessageListParam, SmserMessageLogParam, SmserMessageSendParam,
    SmserNetEaseConfigListParam, SmserNotifyConfigParam, SmserTenConfigListParam,
    SmserTplConfigDeleteParam, SmserTplConfigListParam,
};
#[post("/smser/{method}")]
pub(crate) async fn smser(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    _req: HttpRequest,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "config_add" => {
            smser_config_add(&json_param.param::<SmserConfigAddParam>()?, &auth_dao).await
        }
        "config_del" => {
            smser_config_del(&json_param.param::<SmserConfigDeleteParam>()?, &auth_dao).await
        }
        "config_list" => {
            smser_config_list(&json_param.param::<SmserConfigListParam>()?, &auth_dao).await
        }
        "tpl_config_list" => {
            smser_tpl_config_list(&json_param.param::<SmserTplConfigListParam>()?, &auth_dao).await
        }
        "notify_set_config" => {
            smser_notify_set_config(&json_param.param::<SmserNotifyConfigParam>()?, &auth_dao).await
        }
        "notify_get_config" => smser_notify_get_config(&auth_dao).await,
        "tpl_config_del" => {
            smser_tpl_config_del(&json_param.param::<SmserTplConfigDeleteParam>()?, &auth_dao).await
        }
        "message_send" => {
            smser_message_send(&json_param.param::<SmserMessageSendParam>()?, &auth_dao).await
        }
        "message_list" => {
            smser_message_list(&json_param.param::<SmserMessageListParam>()?, &auth_dao).await
        }
        "message_body" => {
            smser_message_body(&json_param.param::<SmserMessageBodyParam>()?, &auth_dao).await
        }
        "message_cancel" => {
            smser_message_cancel(&json_param.param::<SmserMessageCancelParam>()?, &auth_dao).await
        }
        "message_log" => {
            smser_message_log(&json_param.param::<SmserMessageLogParam>()?, &auth_dao).await
        }
        //ALI短信接口相关接口
        "ali_config_list" => {
            smser_ali_config_list(&json_param.param::<SmserAliConfigListParam>()?, &auth_dao).await
        }
        "ali_app_config_add" => {
            smser_ali_app_config_add(&json_param.param::<SmserAppAliConfigAddParam>()?, &auth_dao)
                .await
        }
        //hw短信接口相关接口
        "hw_config_list" => {
            smser_hw_config_list(&json_param.param::<SmserHwConfigListParam>()?, &auth_dao).await
        }
        "hw_app_config_add" => {
            smser_hw_app_config_add(&json_param.param::<SmserAppHwConfigAddParam>()?, &auth_dao)
                .await
        }
        //腾讯云短信接口相关接口
        "ten_config_list" => {
            smser_ten_config_list(&json_param.param::<SmserTenConfigListParam>()?, &auth_dao).await
        }

        "ten_app_config_add" => {
            smser_ten_app_config_add(&json_param.param::<SmserAppTenConfigAddParam>()?, &auth_dao)
                .await
        }

        //容联短信接口相关接口
        "cloopen_config_list" => {
            smser_cloopen_config_list(
                &json_param.param::<SmserCloOpenConfigListParam>()?,
                &auth_dao,
            )
            .await
        }
        "cloopen_app_config_add" => {
            smser_cloopen_app_config_add(
                &json_param.param::<SmserAppCloopenConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }

        //JD短信接口相关接口
        "jd_config_list" => {
            smser_jd_config_list(&json_param.param::<SmserJDConfigListParam>()?, &auth_dao).await
        }
        "jd_app_config_add" => {
            smser_jd_app_config_add(&json_param.param::<SmserAppJDConfigAddParam>()?, &auth_dao)
                .await
        }

        //网易短信接口相关接口
        "netease_config_list" => {
            smser_netease_config_list(
                &json_param.param::<SmserNetEaseConfigListParam>()?,
                &auth_dao,
            )
            .await
        }
        "netease_app_config_add" => {
            smser_netease_app_config_add(
                &json_param.param::<SmserAppNetEaseConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }

        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
