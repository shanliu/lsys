use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::sender::{
    smser_ali_config_add, smser_ali_config_del, smser_ali_config_edit, smser_ali_config_list,
    smser_app_ali_config_add, smser_app_ali_config_del, smser_app_ali_config_list,
    smser_config_add, smser_config_del, smser_config_list, smser_message_body,
    smser_message_cancel, smser_message_list, smser_message_log, SmserAliConfigAddParam,
    SmserAliConfigDelParam, SmserAliConfigEditParam, SmserAliConfigListParam,
    SmserAppAliConfigAddParam, SmserAppAliConfigDelParam, SmserAppAliConfigListParam,
    SmserConfigAddParam, SmserConfigDeleteParam, SmserConfigListParam, SmserMessageBodyParam,
    SmserMessageCancelParam, SmserMessageListParam, SmserMessageLogParam,
};
#[post("/smser/{method}")]
pub(crate) async fn smser<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "config_add" => smser_config_add(rest.param::<SmserConfigAddParam>()?, &auth_dao).await,
        "config_del" => smser_config_del(rest.param::<SmserConfigDeleteParam>()?, &auth_dao).await,
        "config_list" => smser_config_list(rest.param::<SmserConfigListParam>()?, &auth_dao).await,
        "ali_config_list" => {
            smser_ali_config_list(rest.param::<SmserAliConfigListParam>()?, &auth_dao).await
        }
        "ali_config_add" => {
            smser_ali_config_add(rest.param::<SmserAliConfigAddParam>()?, &auth_dao).await
        }
        "ali_config_edit" => {
            smser_ali_config_edit(rest.param::<SmserAliConfigEditParam>()?, &auth_dao).await
        }
        "ali_config_del" => {
            smser_ali_config_del(rest.param::<SmserAliConfigDelParam>()?, &auth_dao).await
        }
        "ali_app_config_add" => {
            smser_app_ali_config_add(rest.param::<SmserAppAliConfigAddParam>()?, &auth_dao).await
        }
        "ali_app_config_del" => {
            smser_app_ali_config_del(rest.param::<SmserAppAliConfigDelParam>()?, &auth_dao).await
        }
        "ali_app_config_list" => {
            smser_app_ali_config_list(rest.param::<SmserAppAliConfigListParam>()?, &auth_dao).await
        }
        "message_list" => {
            smser_message_list(rest.param::<SmserMessageListParam>()?, &auth_dao).await
        }
        "message_body" => {
            smser_message_body(rest.param::<SmserMessageBodyParam>()?, &auth_dao).await
        }
        "message_cancel" => {
            smser_message_cancel(rest.param::<SmserMessageCancelParam>()?, &auth_dao).await
        }
        "message_log" => smser_message_log(rest.param::<SmserMessageLogParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
