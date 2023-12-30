use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::app::{
    app_add, app_confirm, app_del_parent_app, app_edit, app_list, app_list_sub_app,
    app_list_sub_user, app_reset_secret, app_set_parent_app, app_set_sub_user, app_status,
    app_view_secret, list_parent_app, AppAddParam, AppConfrimParam, AppDelParentAppParam,
    AppEditParam, AppListParam, AppListSubAppParam, AppListSubUserParam, AppResetSecretParam,
    AppSetParentAppParam, AppSetSubUserParam, AppStatusParam, AppViewSecretParam,
    ListParentAppParam,
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
        "status" => app_status(json_param.param::<AppStatusParam>()?, &auth_dao).await,
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
        "set_sub_user" => {
            app_set_sub_user(json_param.param::<AppSetSubUserParam>()?, &auth_dao).await
        }
        "list_sub_user" => {
            app_list_sub_user(json_param.param::<AppListSubUserParam>()?, &auth_dao).await
        }
        "list_sub_app" => {
            app_list_sub_app(json_param.param::<AppListSubAppParam>()?, &auth_dao).await
        }
        "list_parent_app" => {
            list_parent_app(json_param.param::<ListParentAppParam>()?, &auth_dao).await
        }
        "set_parent_app" => {
            app_set_parent_app(json_param.param::<AppSetParentAppParam>()?, &auth_dao).await
        }
        "del_parent_app" => {
            app_del_parent_app(json_param.param::<AppDelParentAppParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
