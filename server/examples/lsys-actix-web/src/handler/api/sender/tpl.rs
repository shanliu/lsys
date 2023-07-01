use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::sender::{
    tpl_body_add, tpl_body_del, tpl_body_edit, tpl_body_list, TplAddParam, TplDelParam,
    TplEditParam, TplListParam,
};
#[post("/tpls/{method}")]
pub(crate) async fn sender_tpl_body<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "add" => tpl_body_add(json_param.param::<TplAddParam>()?, &auth_dao).await,
        "del" => tpl_body_del(json_param.param::<TplDelParam>()?, &auth_dao).await,
        "list" => tpl_body_list(json_param.param::<TplListParam>()?, &auth_dao).await,
        "edit" => tpl_body_edit(json_param.param::<TplEditParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
