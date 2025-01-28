use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::{
    app_list, app_secret_view, AppSecretViewSecretParam, UserAppListParam,
};
// use lsys_web::handler::api::app::{
//     app_reset_secret, app_view_secret, user_app_list, AppResetSecretParam, AppViewSecretParam,
// };

#[post("/{method}")]
pub(crate) async fn app(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        // "status" => app_status(json_param.param::<AppStatusParam>()?, &auth_dao).await,
        // "add" => app_add(json_param.param::<AppAddParam>()?, &auth_dao).await,
        // "edit" => app_edit(json_param.param::<AppEditParam>()?, &auth_dao).await,
        // "confirm" => app_confirm(json_param.param::<AppConfrimParam>()?, &auth_dao).await,
        "user_app_list" => app_list(&json_param.param::<UserAppListParam>()?, &auth_dao).await,
        // "reset_secret" => {
        //     app_secret_view(json_param.param::<AppResetSecretParam>()?, &auth_dao).await
        // }
        "view_secret" => {
            app_secret_view(&json_param.param::<AppSecretViewSecretParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}
