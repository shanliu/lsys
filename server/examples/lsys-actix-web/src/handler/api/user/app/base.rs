use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::app::{
    add, change, confirm, confirm_exter_feature, delete, list_data, oauth_client_request,
    oauth_client_scope_request, oauth_client_setting, oauth_server_client_confirm,
    oauth_server_client_scope_confirm, oauth_server_request, oauth_server_setting,
    request_exter_feature, request_inner_feature_exter_login_request,
    request_inner_feature_sub_app_request, request_list, secret_reset, secret_view,
    sub_app_secret_view, sub_request_list, AddParam, ChangeParam, ConfirmExterFeatureParam,
    ConfirmOAuthClientParam, ConfirmOAuthClientScopeParam, ConfirmOAuthClientSettingParam,
    ConfirmOAuthServerSettingParam, ConfirmParam, DeleteParam, OAuthClientRequestParam,
    OAuthServerRequestData, RequestExterFeatureParam, RequestExterLoginFeatureData,
    RequestExterSubAppData, RequestListParam, ResetSecretParam, SecretViewSecretParam,
    SubRequestListParam, UserAppListParam,
};
#[post("/{method}")]
pub(crate) async fn base(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "add" => add(&json_param.param::<AddParam>()?, &auth_dao).await,
        "change" => change(&json_param.param::<ChangeParam>()?, &auth_dao).await,
        "secret_reset" => secret_reset(&json_param.param::<ResetSecretParam>()?, &auth_dao).await,
        "secret_view" => {
            secret_view(&json_param.param::<SecretViewSecretParam>()?, &auth_dao).await
        }
        "sub_app_secret_view" => {
            sub_app_secret_view(&json_param.param::<SecretViewSecretParam>()?, &auth_dao).await
        }
        "confirm" => confirm(&json_param.param::<ConfirmParam>()?, &auth_dao).await,
        "list" => list_data(&json_param.param::<UserAppListParam>()?, &auth_dao).await,
        "request_list" => request_list(&json_param.param::<RequestListParam>()?, &auth_dao).await,
        "sub_request_list" => {
            sub_request_list(&json_param.param::<SubRequestListParam>()?, &auth_dao).await
        }
        "delete" => delete(&json_param.param::<DeleteParam>()?, &auth_dao).await,
        "request_exter_feature" => {
            request_exter_feature(&json_param.param::<RequestExterFeatureParam>()?, &auth_dao).await
        }
        "confirm_exter_feature" => {
            confirm_exter_feature(&json_param.param::<ConfirmExterFeatureParam>()?, &auth_dao).await
        }
        "request_inner_feature_exter_login_request" => {
            request_inner_feature_exter_login_request(
                &json_param.param::<RequestExterLoginFeatureData>()?,
                &auth_dao,
            )
            .await
        }
        "request_inner_feature_sub_app_request" => {
            request_inner_feature_sub_app_request(
                &json_param.param::<RequestExterSubAppData>()?,
                &auth_dao,
            )
            .await
        }
        "oauth_client_request" => {
            oauth_client_request(&json_param.param::<OAuthClientRequestParam>()?, &auth_dao).await
        }
        "oauth_client_scope_request" => {
            oauth_client_scope_request(&json_param.param::<OAuthClientRequestParam>()?, &auth_dao)
                .await
        }
        "oauth_client_setting" => {
            oauth_client_setting(
                &json_param.param::<ConfirmOAuthClientSettingParam>()?,
                &auth_dao,
            )
            .await
        }
        "oauth_server_client_confirm" => {
            oauth_server_client_confirm(&json_param.param::<ConfirmOAuthClientParam>()?, &auth_dao)
                .await
        }
        "oauth_server_client_scope_confirm" => {
            oauth_server_client_scope_confirm(
                &json_param.param::<ConfirmOAuthClientScopeParam>()?,
                &auth_dao,
            )
            .await
        }
        "oauth_server_request" => {
            oauth_server_request(&json_param.param::<OAuthServerRequestData>()?, &auth_dao).await
        }
        "oauth_server_setting" => {
            oauth_server_setting(
                &json_param.param::<ConfirmOAuthServerSettingParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_data(&e))?
    .into())
}
