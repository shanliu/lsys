use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::user::app::{
    add, app_list, app_secret_add, app_secret_change, app_secret_del, change, confirm,
    confirm_exter_feature, delete, mapping_data, notify_secret_change, oauth_client_request,
    oauth_client_scope_request, oauth_client_set_domain, oauth_secret_add, oauth_secret_change,
    oauth_secret_del, oauth_server_client_confirm, oauth_server_client_scope_confirm,
    oauth_server_request, oauth_server_setting, parent_app_list, request_exter_feature,
    request_inner_feature_exter_login_request, request_list, secret_view,
    sub_app_notify_get_config, sub_app_notify_set_config, sub_app_request, sub_app_secret_view,
    sub_request_list, AddAppSecretParam, AddOAuthSecretParam, AddParam, ChangeAppSecretParam,
    ChangeNotifySecretParam, ChangeOAuthSecretParam, ChangeParam, ConfirmExterFeatureParam,
    ConfirmOAuthClientParam, ConfirmOAuthClientScopeParam, ConfirmOAuthClientSetDomainParam,
    ConfirmOAuthServerSettingParam, ConfirmParam, DelAppSecretParam, DelOAuthSecretParam,
    DeleteParam, OAuthClientRequestParam, OAuthServerRequestParam, RequestExterFeatureParam,
    RequestExterLoginFeatureParam, RequestExterSubAppParam, RequestListParam,
    SecretViewSecretParam, SubAppNotifyConfigParam, SubRequestListParam, UserAppListParam,
    UserParentAppListParam,
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
        "mapping" => mapping_data(&auth_dao).await,
        "parent_app" => {
            parent_app_list(&json_param.param::<UserParentAppListParam>()?, &auth_dao).await
        }
        "add" => add(&json_param.param::<AddParam>()?, &auth_dao).await,
        "confirm" => confirm(&json_param.param::<ConfirmParam>()?, &auth_dao).await,
        "change" => change(&json_param.param::<ChangeParam>()?, &auth_dao).await,
        "app_secret_add" => {
            app_secret_add(&json_param.param::<AddAppSecretParam>()?, &auth_dao).await
        }
        "app_secret_change" => {
            app_secret_change(&json_param.param::<ChangeAppSecretParam>()?, &auth_dao).await
        }
        "app_secret_del" => {
            app_secret_del(&json_param.param::<DelAppSecretParam>()?, &auth_dao).await
        }
        "notify_secret_change" => {
            notify_secret_change(&json_param.param::<ChangeNotifySecretParam>()?, &auth_dao).await
        }
        "secret_view" => {
            secret_view(&json_param.param::<SecretViewSecretParam>()?, &auth_dao).await
        }
        "sub_app_secret_view" => {
            sub_app_secret_view(&json_param.param::<SecretViewSecretParam>()?, &auth_dao).await
        }
        "list" => app_list(&json_param.param::<UserAppListParam>()?, &auth_dao).await,
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
                &json_param.param::<RequestExterLoginFeatureParam>()?,
                &auth_dao,
            )
            .await
        }
        "sub_app_request" => {
            sub_app_request(&json_param.param::<RequestExterSubAppParam>()?, &auth_dao).await
        }
        "sub_app_notify_get_config" => sub_app_notify_get_config(&auth_dao).await,
        "sub_app_notify_set_config" => {
            sub_app_notify_set_config(&json_param.param::<SubAppNotifyConfigParam>()?, &auth_dao)
                .await
        }

        "oauth_client_request" => {
            oauth_client_request(&json_param.param::<OAuthClientRequestParam>()?, &auth_dao).await
        }
        "oauth_client_scope_request" => {
            oauth_client_scope_request(&json_param.param::<OAuthClientRequestParam>()?, &auth_dao)
                .await
        }
        "oauth_client_set_domain" => {
            oauth_client_set_domain(
                &json_param.param::<ConfirmOAuthClientSetDomainParam>()?,
                &auth_dao,
            )
            .await
        }
        "oauth_client_secret_add" => {
            oauth_secret_add(&json_param.param::<AddOAuthSecretParam>()?, &auth_dao).await
        }
        "oauth_client_secret_change" => {
            oauth_secret_change(&json_param.param::<ChangeOAuthSecretParam>()?, &auth_dao).await
        }
        "oauth_client_secret_del" => {
            oauth_secret_del(&json_param.param::<DelOAuthSecretParam>()?, &auth_dao).await
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
            oauth_server_request(&json_param.param::<OAuthServerRequestParam>()?, &auth_dao).await
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
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
