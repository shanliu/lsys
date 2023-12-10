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
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let (path,): (String,) = path.into_inner();
    Ok(match path.as_str() {
        "add" => app_add(json_param.param::<AppAddParam>()?, &auth_dao).await,
        "edit" => app_edit(json_param.param::<AppEditParam>()?, &auth_dao).await,
        "confirm" => app_confirm(json_param.param::<AppConfrimParam>()?, &auth_dao).await,
        "list" => app_list(json_param.param::<AppListParam>()?, &auth_dao).await,
        "reset_secret" => {
            app_reset_secret(json_param.param::<AppResetSecretParam>()?, &auth_dao).await
        }
        "view_secret" => {
            app_view_secret(json_param.param::<AppViewSecretParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
