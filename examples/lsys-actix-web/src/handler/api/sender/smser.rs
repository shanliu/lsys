use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::sender::{
    smser_ali_config_add, smser_ali_config_del, smser_ali_config_edit, smser_ali_config_list,
    smser_app_ali_config_add, smser_app_ali_config_del, smser_config_add, smser_config_del,
    smser_config_list, SmserAliConfigAddParam, SmserAliConfigDelParam, SmserAliConfigEditParam,
    SmserAliConfigListParam, SmserAppAliConfigAddParam, SmserAppAliConfigDelParam,
    SmserConfigAddParam, SmserConfigDeleteParam, SmserConfigListParam,
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
        name => handler_not_found!(name),
    }?
    .into())
}