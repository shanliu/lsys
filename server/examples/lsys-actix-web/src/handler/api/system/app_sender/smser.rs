use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::{post, HttpRequest};
use lsys_web::handler::api::system::app_sender::{
    smser_ali_config_add, smser_ali_config_del, smser_ali_config_edit, smser_ali_config_list,
    smser_message_body, smser_message_cancel, smser_message_list, smser_message_log,
    smser_tpl_config_ali_add, SmserAliConfigAddParam, SmserAliConfigDelParam,
    SmserAliConfigEditParam, SmserAliConfigListParam, SmserAppAliConfigAddParam,
    SmserMessageBodyParam, SmserMessageCancelParam, SmserMessageListParam, SmserMessageLogParam,
};
use lsys_web::handler::api::system::app_sender::{
    smser_cloopen_config_add, smser_cloopen_config_del, smser_cloopen_config_edit,
    smser_cloopen_config_list, smser_tpl_config_cloopen_add, SmserAppCloopenConfigAddParam,
    SmserCloOpenConfigAddParam, SmserCloOpenConfigDelParam, SmserCloOpenConfigEditParam,
    SmserCloOpenConfigListParam,
};
use lsys_web::handler::api::system::app_sender::{
    smser_config_add, smser_config_del, smser_config_list, smser_tpl_config_del,
    smser_tpl_config_list, SmserConfigAddParam, SmserConfigDeleteParam, SmserConfigListParam,
    SmserTplConfigDelParam, SmserTplConfigListParam,
};
use lsys_web::handler::api::system::app_sender::{
    smser_hw_config_add, smser_hw_config_del, smser_hw_config_edit, smser_hw_config_list,
    smser_tpl_config_hw_add, SmserAppHwConfigAddParam, SmserHwConfigAddParam,
    SmserHwConfigDelParam, SmserHwConfigEditParam, SmserHwConfigListParam,
};
use lsys_web::handler::api::system::app_sender::{
    smser_jd_config_add, smser_jd_config_del, smser_jd_config_edit, smser_jd_config_list,
    smser_tpl_config_jd_add, SmserAppJDConfigAddParam, SmserJDConfigAddParam,
    SmserJDConfigDelParam, SmserJDConfigEditParam, SmserJDConfigListParam,
};
use lsys_web::handler::api::system::app_sender::{
    smser_netease_config_add, smser_netease_config_del, smser_netease_config_edit,
    smser_netease_config_list, smser_tpl_config_netease_add, SmserAppNetEaseConfigAddParam,
    SmserNetEaseConfigAddParam, SmserNetEaseConfigDelParam, SmserNetEaseConfigEditParam,
    SmserNetEaseConfigListParam,
};
use lsys_web::handler::api::system::app_sender::{
    smser_ten_config_add, smser_ten_config_del, smser_ten_config_edit, smser_ten_config_list,
    smser_tpl_config_ten_add, SmserAppTenConfigAddParam, SmserTenConfigAddParam,
    SmserTenConfigDelParam, SmserTenConfigEditParam, SmserTenConfigListParam,
};

#[post("smser/{method}")]
pub(crate) async fn smser(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req: HttpRequest,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "message_logs" => {
            smser_message_log(&json_param.param::<SmserMessageLogParam>()?, &auth_dao).await
        }
        "message_view" => {
            smser_message_body(&json_param.param::<SmserMessageBodyParam>()?, &auth_dao).await
        }
        "message_list" => {
            smser_message_list(&json_param.param::<SmserMessageListParam>()?, &auth_dao).await
        }
        "message_cancel" => {
            smser_message_cancel(&json_param.param::<SmserMessageCancelParam>()?, &auth_dao).await
        }

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
        "tpl_config_delete" => {
            smser_tpl_config_del(&json_param.param::<SmserTplConfigDelParam>()?, &auth_dao).await
        }
        //ali
        "ali_config_list" => {
            smser_ali_config_list(
                &json_param.param::<SmserAliConfigListParam>()?,
                |key| {
                    req.url_for(
                        "sms_notify",
                        [key.model().id.to_string(), key.callback_key.to_owned()],
                    )
                    .map(|e| e.to_string())
                    .unwrap_or_default()
                },
                &auth_dao,
            )
            .await
        }
        "ali_config_add" => {
            smser_ali_config_add(&json_param.param::<SmserAliConfigAddParam>()?, &auth_dao).await
        }
        "ali_config_edit" => {
            smser_ali_config_edit(&json_param.param::<SmserAliConfigEditParam>()?, &auth_dao).await
        }
        "ali_config_del" => {
            smser_ali_config_del(&json_param.param::<SmserAliConfigDelParam>()?, &auth_dao).await
        }
        "ali_tpl_config_add" => {
            smser_tpl_config_ali_add(&json_param.param::<SmserAppAliConfigAddParam>()?, &auth_dao)
                .await
        }
        //cloopen
        "cloopen_config_list" => {
            smser_cloopen_config_list(
                &json_param.param::<SmserCloOpenConfigListParam>()?,
                |key| {
                    req.url_for(
                        "sms_notify",
                        [key.model().id.to_string(), key.callback_key.to_owned()],
                    )
                    .map(|e| e.to_string())
                    .unwrap_or_default()
                },
                &auth_dao,
            )
            .await
        }
        "cloopen_config_add" => {
            smser_cloopen_config_add(
                &json_param.param::<SmserCloOpenConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }
        "cloopen_config_edit" => {
            smser_cloopen_config_edit(
                &json_param.param::<SmserCloOpenConfigEditParam>()?,
                &auth_dao,
            )
            .await
        }
        "cloopen_config_del" => {
            smser_cloopen_config_del(
                &json_param.param::<SmserCloOpenConfigDelParam>()?,
                &auth_dao,
            )
            .await
        }
        "cloopen_tpl_config_add" => {
            smser_tpl_config_cloopen_add(
                &json_param.param::<SmserAppCloopenConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }
        //hw
        "hw_config_list" => {
            smser_hw_config_list(
                &json_param.param::<SmserHwConfigListParam>()?,
                |key| {
                    req.url_for(
                        "sms_notify",
                        [key.model().id.to_string(), key.callback_key.to_owned()],
                    )
                    .map(|e| e.to_string())
                    .unwrap_or_default()
                },
                &auth_dao,
            )
            .await
        }
        "hw_config_add" => {
            smser_hw_config_add(&json_param.param::<SmserHwConfigAddParam>()?, &auth_dao).await
        }
        "hw_config_edit" => {
            smser_hw_config_edit(&json_param.param::<SmserHwConfigEditParam>()?, &auth_dao).await
        }
        "hw_config_del" => {
            smser_hw_config_del(&json_param.param::<SmserHwConfigDelParam>()?, &auth_dao).await
        }
        "hw_tpl_config_add" => {
            smser_tpl_config_hw_add(&json_param.param::<SmserAppHwConfigAddParam>()?, &auth_dao)
                .await
        }
        //netease
        "netease_config_list" => {
            smser_netease_config_list(
                &json_param.param::<SmserNetEaseConfigListParam>()?,
                |key| {
                    req.url_for("sms_notify", [key.model().id.to_string(), "".to_string()])
                        .map(|e| e.to_string())
                        .unwrap_or_default()
                },
                &auth_dao,
            )
            .await
        }
        "netease_config_add" => {
            smser_netease_config_add(
                &json_param.param::<SmserNetEaseConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }
        "netease_config_edit" => {
            smser_netease_config_edit(
                &json_param.param::<SmserNetEaseConfigEditParam>()?,
                &auth_dao,
            )
            .await
        }
        "netease_config_del" => {
            smser_netease_config_del(
                &json_param.param::<SmserNetEaseConfigDelParam>()?,
                &auth_dao,
            )
            .await
        }
        "netease_tpl_config_add" => {
            smser_tpl_config_netease_add(
                &json_param.param::<SmserAppNetEaseConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }
        //jd
        "jd_config_list" => {
            smser_jd_config_list(&json_param.param::<SmserJDConfigListParam>()?, &auth_dao).await
        }
        "jd_config_add" => {
            smser_jd_config_add(&json_param.param::<SmserJDConfigAddParam>()?, &auth_dao).await
        }
        "jd_config_edit" => {
            smser_jd_config_edit(&json_param.param::<SmserJDConfigEditParam>()?, &auth_dao).await
        }
        "jd_config_del" => {
            smser_jd_config_del(&json_param.param::<SmserJDConfigDelParam>()?, &auth_dao).await
        }
        "jd_tpl_config_add" => {
            smser_tpl_config_jd_add(&json_param.param::<SmserAppJDConfigAddParam>()?, &auth_dao)
                .await
        }
        //tencent
        "tencent_config_list" => {
            smser_ten_config_list(
                &json_param.param::<SmserTenConfigListParam>()?,
                |key| {
                    req.url_for(
                        "sms_notify",
                        [key.model().id.to_string(), key.callback_key.to_owned()],
                    )
                    .map(|e| e.to_string())
                    .unwrap_or_default()
                },
                &auth_dao,
            )
            .await
        }
        "tencent_config_add" => {
            smser_ten_config_add(&json_param.param::<SmserTenConfigAddParam>()?, &auth_dao).await
        }
        "tencent_config_edit" => {
            smser_ten_config_edit(&json_param.param::<SmserTenConfigEditParam>()?, &auth_dao).await
        }
        "tencent_config_del" => {
            smser_ten_config_del(&json_param.param::<SmserTenConfigDelParam>()?, &auth_dao).await
        }
        "tencent_tpl_config_add" => {
            smser_tpl_config_ten_add(&json_param.param::<SmserAppTenConfigAddParam>()?, &auth_dao)
                .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
