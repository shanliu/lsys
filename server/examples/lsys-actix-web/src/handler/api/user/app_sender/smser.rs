use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::{HttpRequest, post};
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::app_sender::{
    SmserAliConfigListParam, SmserAppAliConfigAddParam, SmserAppCloopenConfigAddParam,
    SmserAppHwConfigAddParam, SmserAppJDConfigAddParam, SmserAppNetEaseConfigAddParam,
    SmserAppTenConfigAddParam, SmserCloOpenConfigListParam, SmserConfigAddParam,
    SmserConfigDeleteParam, SmserConfigListParam, SmserHwConfigListParam, SmserJDConfigListParam,
    SmserMessageBodyParam, SmserMessageCancelParam, SmserMessageListParam, SmserMessageLogParam,
    SmserMessageSendParam, SmserNetEaseConfigListParam, SmserNotifyConfigParam,
    SmserTenConfigListParam, SmserTplConfigDeleteParam, SmserTplConfigListParam,
    smser_ali_app_config_add, smser_ali_config_list, smser_cloopen_app_config_add,
    smser_cloopen_config_list, smser_config_add, smser_config_del, smser_config_list,
    smser_hw_app_config_add, smser_hw_config_list, smser_jd_app_config_add, smser_jd_config_list,
    smser_mapping_data, smser_message_body, smser_message_cancel, smser_message_list,
    smser_message_log, smser_message_send, smser_netease_app_config_add, smser_netease_config_list,
    smser_notify_get_config, smser_notify_set_config, smser_ten_app_config_add,
    smser_ten_config_list, smser_tpl_config_del, smser_tpl_config_list,
};
#[post("/smser/{method}")]
pub(crate) async fn smser(
    bearer: BearerQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
    _req: HttpRequest,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => smser_mapping_data(&req_query).await,
        "config_add" => {
            smser_config_add(&json_param.param::<SmserConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "config_del" => {
            smser_config_del(&json_param.param::<SmserConfigDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "config_list" => {
            smser_config_list(&json_param.param::<SmserConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tpl_config_list" => {
            smser_tpl_config_list(&json_param.param::<SmserTplConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tpl_config_del" => {
            smser_tpl_config_del(&json_param.param::<SmserTplConfigDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_send" => {
            smser_message_send(&json_param.param::<SmserMessageSendParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_list" => {
            smser_message_list(&json_param.param::<SmserMessageListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_view" => {
            smser_message_body(&json_param.param::<SmserMessageBodyParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_cancel" => {
            smser_message_cancel(&json_param.param::<SmserMessageCancelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_logs" => {
            smser_message_log(&json_param.param::<SmserMessageLogParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "notify_set_config" => {
            smser_notify_set_config(&json_param.param::<SmserNotifyConfigParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "notify_get_config" => smser_notify_get_config(&req_query, &auth_dao, web_dao.as_ref()).await,
        //ALI短信接口相关接口
        "ali_config_list" => {
            smser_ali_config_list(&json_param.param::<SmserAliConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "ali_app_config_add" => {
            smser_ali_app_config_add(&json_param.param::<SmserAppAliConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        //hw短信接口相关接口
        "hw_config_list" => {
            smser_hw_config_list(&json_param.param::<SmserHwConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "hw_app_config_add" => {
            smser_hw_app_config_add(&json_param.param::<SmserAppHwConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        //腾讯云短信接口相关接口
        "ten_config_list" => {
            smser_ten_config_list(&json_param.param::<SmserTenConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }

        "ten_app_config_add" => {
            smser_ten_app_config_add(&json_param.param::<SmserAppTenConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }

        //容联短信接口相关接口
        "cloopen_config_list" => {
            smser_cloopen_config_list(
                &json_param.param::<SmserCloOpenConfigListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "cloopen_app_config_add" => {
            smser_cloopen_app_config_add(
                &json_param.param::<SmserAppCloopenConfigAddParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }

        //JD短信接口相关接口
        "jd_config_list" => {
            smser_jd_config_list(&json_param.param::<SmserJDConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "jd_app_config_add" => {
            smser_jd_app_config_add(&json_param.param::<SmserAppJDConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }

        //网易短信接口相关接口
        "netease_config_list" => {
            smser_netease_config_list(
                &json_param.param::<SmserNetEaseConfigListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "netease_app_config_add" => {
            smser_netease_app_config_add(
                &json_param.param::<SmserAppNetEaseConfigAddParam>()?,
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
