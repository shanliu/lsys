use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::sender::{
    smser_ali_config_add, smser_ali_config_del, smser_ali_config_edit, smser_ali_config_list,
    smser_config_add, smser_config_add_ali, smser_config_add_hw, smser_config_add_ten,
    smser_config_del, smser_config_list, smser_hw_config_add, smser_hw_config_del,
    smser_hw_config_edit, smser_hw_config_list, smser_message_body, smser_message_cancel,
    smser_message_list, smser_message_log, smser_message_send, smser_ten_config_add,
    smser_ten_config_del, smser_ten_config_edit, smser_ten_config_list, smser_tpl_config_del,
    smser_tpl_config_list, SmserAliConfigAddParam, SmserAliConfigDelParam, SmserAliConfigEditParam,
    SmserAliConfigListParam, SmserAppAliConfigAddParam, SmserAppHwConfigAddParam,
    SmserAppTenConfigAddParam, SmserConfigAddParam, SmserConfigDeleteParam, SmserConfigListParam,
    SmserHwConfigAddParam, SmserHwConfigDelParam, SmserHwConfigEditParam, SmserHwConfigListParam,
    SmserMessageBodyParam, SmserMessageCancelParam, SmserMessageListParam, SmserMessageLogParam,
    SmserMessageSendParam, SmserTenConfigAddParam, SmserTenConfigDelParam, SmserTenConfigEditParam,
    SmserTenConfigListParam, TplConfigDelParam, TplConfigListParam,
};
#[post("/smser/{method}")]
pub(crate) async fn sender_smser<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "config_add" => {
            smser_config_add(json_param.param::<SmserConfigAddParam>()?, &auth_dao).await
        }
        "config_del" => {
            smser_config_del(json_param.param::<SmserConfigDeleteParam>()?, &auth_dao).await
        }
        "config_list" => {
            smser_config_list(json_param.param::<SmserConfigListParam>()?, &auth_dao).await
        }
        "tpl_config_list" => {
            smser_tpl_config_list(json_param.param::<TplConfigListParam>()?, &auth_dao).await
        }
        "tpl_config_del" => {
            smser_tpl_config_del(json_param.param::<TplConfigDelParam>()?, &auth_dao).await
        }
        "message_send" => {
            smser_message_send(json_param.param::<SmserMessageSendParam>()?, &auth_dao).await
        }
        "message_list" => {
            smser_message_list(json_param.param::<SmserMessageListParam>()?, &auth_dao).await
        }
        "message_body" => {
            smser_message_body(json_param.param::<SmserMessageBodyParam>()?, &auth_dao).await
        }
        "message_cancel" => {
            smser_message_cancel(json_param.param::<SmserMessageCancelParam>()?, &auth_dao).await
        }
        "message_log" => {
            smser_message_log(json_param.param::<SmserMessageLogParam>()?, &auth_dao).await
        }
        //ALI短信接口相关接口
        "ali_config_list" => {
            smser_ali_config_list(json_param.param::<SmserAliConfigListParam>()?, &auth_dao).await
        }
        "ali_config_add" => {
            smser_ali_config_add(json_param.param::<SmserAliConfigAddParam>()?, &auth_dao).await
        }
        "ali_config_edit" => {
            smser_ali_config_edit(json_param.param::<SmserAliConfigEditParam>()?, &auth_dao).await
        }
        "ali_config_del" => {
            smser_ali_config_del(json_param.param::<SmserAliConfigDelParam>()?, &auth_dao).await
        }
        "ali_app_config_add" => {
            smser_config_add_ali(json_param.param::<SmserAppAliConfigAddParam>()?, &auth_dao).await
        }
        //hw短信接口相关接口
        "hw_config_list" => {
            smser_hw_config_list(json_param.param::<SmserHwConfigListParam>()?, &auth_dao).await
        }
        "hw_config_add" => {
            smser_hw_config_add(json_param.param::<SmserHwConfigAddParam>()?, &auth_dao).await
        }
        "hw_config_edit" => {
            smser_hw_config_edit(json_param.param::<SmserHwConfigEditParam>()?, &auth_dao).await
        }
        "hw_config_del" => {
            smser_hw_config_del(json_param.param::<SmserHwConfigDelParam>()?, &auth_dao).await
        }
        "hw_app_config_add" => {
            smser_config_add_hw(json_param.param::<SmserAppHwConfigAddParam>()?, &auth_dao).await
        }
        //hw短信接口相关接口
        "ten_config_list" => {
            smser_ten_config_list(json_param.param::<SmserTenConfigListParam>()?, &auth_dao).await
        }
        "ten_config_add" => {
            smser_ten_config_add(json_param.param::<SmserTenConfigAddParam>()?, &auth_dao).await
        }
        "ten_config_edit" => {
            smser_ten_config_edit(json_param.param::<SmserTenConfigEditParam>()?, &auth_dao).await
        }
        "ten_config_del" => {
            smser_ten_config_del(json_param.param::<SmserTenConfigDelParam>()?, &auth_dao).await
        }
        "ten_app_config_add" => {
            smser_config_add_ten(json_param.param::<SmserAppTenConfigAddParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
