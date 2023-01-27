use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::app::{
    app_add, app_confirm, app_edit, app_list, app_reset_secret, app_view_secret, AppAddParam,
    AppConfrimParam, AppEditParam, AppListParam, AppResetSecretParam, AppViewSecretParam,
};

#[post("/{method}")]
pub(crate) async fn app<'t>(
    jwt: JwtQuery,
    path: actix_web::web::Path<(String,)>,
    rest: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.0.to_string().as_str() {
        "add" => app_add(rest.param::<AppAddParam>()?, &auth_dao).await,
        "edit" => app_edit(rest.param::<AppEditParam>()?, &auth_dao).await,
        "confirm" => app_confirm(rest.param::<AppConfrimParam>()?, &auth_dao).await,
        "list" => app_list(rest.param::<AppListParam>()?, &auth_dao).await,
        "reset_secret" => app_reset_secret(rest.param::<AppResetSecretParam>()?, &auth_dao).await,
        "view_secret" => app_view_secret(rest.param::<AppViewSecretParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    }?
    .into())
}
