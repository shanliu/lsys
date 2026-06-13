use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::{HttpRequest, post};
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::app_sender::{
    SmserAliConfigAddParam, SmserAliConfigDelParam, SmserAliConfigEditParam,
    SmserAliConfigListParam, SmserAppAliConfigAddParam, SmserMessageBodyParam,
    SmserMessageCancelParam, SmserMessageListParam, SmserMessageLogParam, smser_ali_config_add,
    smser_ali_config_del, smser_ali_config_edit, smser_ali_config_list, smser_mapping_data,
    smser_message_body, smser_message_cancel, smser_message_list, smser_message_log,
    smser_tpl_config_ali_add,
};
use lsys_web::handler::api::system::app_sender::{
    SmserAppCloopenConfigAddParam, SmserCloOpenConfigAddParam, SmserCloOpenConfigDelParam,
    SmserCloOpenConfigEditParam, SmserCloOpenConfigListParam, smser_cloopen_config_add,
    smser_cloopen_config_del, smser_cloopen_config_edit, smser_cloopen_config_list,
    smser_tpl_config_cloopen_add,
};
use lsys_web::handler::api::system::app_sender::{
    SmserAppEmayConfigAddParam, SmserEmayConfigAddParam, SmserEmayConfigDelParam,
    SmserEmayConfigEditParam, SmserEmayConfigListParam, smser_emay_config_add,
    smser_emay_config_del, smser_emay_config_edit, smser_emay_config_list,
    smser_tpl_config_emay_add,
};
use lsys_web::handler::api::system::app_sender::{
    SmserAppHwConfigAddParam, SmserHwConfigAddParam, SmserHwConfigDelParam, SmserHwConfigEditParam,
    SmserHwConfigListParam, smser_hw_config_add, smser_hw_config_del, smser_hw_config_edit,
    smser_hw_config_list, smser_tpl_config_hw_add,
};
use lsys_web::handler::api::system::app_sender::{
    SmserAppJDConfigAddParam, SmserJDConfigAddParam, SmserJDConfigDelParam, SmserJDConfigEditParam,
    SmserJDConfigListParam, smser_jd_config_add, smser_jd_config_del, smser_jd_config_edit,
    smser_jd_config_list, smser_tpl_config_jd_add,
};
use lsys_web::handler::api::system::app_sender::{
    SmserAppNetEaseConfigAddParam, SmserNetEaseConfigAddParam, SmserNetEaseConfigDelParam,
    SmserNetEaseConfigEditParam, SmserNetEaseConfigListParam, smser_netease_config_add,
    smser_netease_config_del, smser_netease_config_edit, smser_netease_config_list,
    smser_tpl_config_netease_add,
};
use lsys_web::handler::api::system::app_sender::{
    SmserAppTenConfigAddParam, SmserTenConfigAddParam, SmserTenConfigDelParam,
    SmserTenConfigEditParam, SmserTenConfigListParam, smser_ten_config_add, smser_ten_config_del,
    smser_ten_config_edit, smser_ten_config_list, smser_tpl_config_ten_add,
};
use lsys_web::handler::api::system::app_sender::{
    SmserConfigAddParam, SmserConfigDeleteParam, SmserConfigListParam, SmserTplConfigDelParam,
    SmserTplConfigListParam, smser_config_add, smser_config_del, smser_config_list,
    smser_tpl_config_del, smser_tpl_config_list,
};

#[post("smser/{method}")]
pub(crate) async fn smser(
    bearer: BearerQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req: HttpRequest,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => smser_mapping_data(&req_query).await,
        "message_logs" => {
            smser_message_log(&json_param.param::<SmserMessageLogParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_view" => {
            smser_message_body(&json_param.param::<SmserMessageBodyParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_list" => {
            smser_message_list(&json_param.param::<SmserMessageListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "message_cancel" => {
            smser_message_cancel(&json_param.param::<SmserMessageCancelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }

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
        "tpl_config_delete" => {
            smser_tpl_config_del(&json_param.param::<SmserTplConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
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
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "ali_config_add" => {
            smser_ali_config_add(&json_param.param::<SmserAliConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "ali_config_edit" => {
            smser_ali_config_edit(&json_param.param::<SmserAliConfigEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "ali_config_del" => {
            smser_ali_config_del(&json_param.param::<SmserAliConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "ali_tpl_config_add" => {
            smser_tpl_config_ali_add(&json_param.param::<SmserAppAliConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
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
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "cloopen_config_add" => {
            smser_cloopen_config_add(
                &json_param.param::<SmserCloOpenConfigAddParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "cloopen_config_edit" => {
            smser_cloopen_config_edit(
                &json_param.param::<SmserCloOpenConfigEditParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "cloopen_config_del" => {
            smser_cloopen_config_del(
                &json_param.param::<SmserCloOpenConfigDelParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "cloopen_tpl_config_add" => {
            smser_tpl_config_cloopen_add(
                &json_param.param::<SmserAppCloopenConfigAddParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
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
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "hw_config_add" => {
            smser_hw_config_add(&json_param.param::<SmserHwConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "hw_config_edit" => {
            smser_hw_config_edit(&json_param.param::<SmserHwConfigEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "hw_config_del" => {
            smser_hw_config_del(&json_param.param::<SmserHwConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "hw_tpl_config_add" => {
            smser_tpl_config_hw_add(&json_param.param::<SmserAppHwConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
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
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "netease_config_add" => {
            smser_netease_config_add(
                &json_param.param::<SmserNetEaseConfigAddParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "netease_config_edit" => {
            smser_netease_config_edit(
                &json_param.param::<SmserNetEaseConfigEditParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "netease_config_del" => {
            smser_netease_config_del(
                &json_param.param::<SmserNetEaseConfigDelParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "netease_tpl_config_add" => {
            smser_tpl_config_netease_add(
                &json_param.param::<SmserAppNetEaseConfigAddParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        //jd
        "jd_config_list" => {
            smser_jd_config_list(&json_param.param::<SmserJDConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "jd_config_add" => {
            smser_jd_config_add(&json_param.param::<SmserJDConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "jd_config_edit" => {
            smser_jd_config_edit(&json_param.param::<SmserJDConfigEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "jd_config_del" => {
            smser_jd_config_del(&json_param.param::<SmserJDConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "jd_tpl_config_add" => {
            smser_tpl_config_jd_add(&json_param.param::<SmserAppJDConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
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
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "tencent_config_add" => {
            smser_ten_config_add(&json_param.param::<SmserTenConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tencent_config_edit" => {
            smser_ten_config_edit(&json_param.param::<SmserTenConfigEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tencent_config_del" => {
            smser_ten_config_del(&json_param.param::<SmserTenConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "tencent_tpl_config_add" => {
            smser_tpl_config_ten_add(&json_param.param::<SmserAppTenConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        //emay
        "emay_config_list" => {
            smser_emay_config_list(
                &json_param.param::<SmserEmayConfigListParam>()?,
                |key| {
                    req.url_for(
                        "sms_notify",
                        [key.model().id.to_string(), key.callback_key.to_owned()],
                    )
                    .map(|e| e.to_string())
                    .unwrap_or_default()
                },
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "emay_config_add" => {
            smser_emay_config_add(&json_param.param::<SmserEmayConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "emay_config_edit" => {
            smser_emay_config_edit(&json_param.param::<SmserEmayConfigEditParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "emay_config_del" => {
            smser_emay_config_del(&json_param.param::<SmserEmayConfigDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "emay_tpl_config_add" => {
            smser_tpl_config_emay_add(
                &json_param.param::<SmserAppEmayConfigAddParam>()?,
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
