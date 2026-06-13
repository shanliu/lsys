use crate::common::handler::{
    JsonQuery, BearerQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::user::app::{
    AddAppSecretParam, AddOAuthSecretParam, AddParam, AppStatParam, ChangeAppSecretParam,
    ChangeNotifySecretParam, ChangeOAuthSecretParam, ChangeParam, ConfirmExterFeatureParam,
    ConfirmOAuthClientParam, ConfirmOAuthClientScopeParam, ConfirmOAuthClientSetDomainParam,
    ConfirmOAuthServerSettingParam, ConfirmParam, DelAppSecretParam, DelOAuthSecretParam,
    DeleteParam, NotifyDataDelParam, NotifyDataListParam, OAuthClientRequestParam,
    OAuthClientScopeDataParam, OAuthServerRequestParam, RequestExterFeatureParam,
    RequestExterLoginFeatureParam, RequestExterSubAppParam, RequestListParam,
    SecretViewSecretParam, SubAppListParam, SubAppNotifyGetConfigParam, SubAppNotifySetConfigParam,
    SubRequestListParam, UserAppListParam, UserParentAppListParam, add, app_list, app_secret_add,
    app_secret_change, app_secret_del, change, confirm, confirm_exter_feature, delete,
    mapping_data, notify_data_del, notify_data_list, notify_secret_change, oauth_client_request,
    oauth_client_scope_data, oauth_client_scope_request, oauth_client_set_domain, oauth_secret_add,
    oauth_secret_change, oauth_secret_del, oauth_server_client_confirm,
    oauth_server_client_scope_confirm, oauth_server_request, oauth_server_setting, parent_app_list,
    request_exter_feature, request_inner_feature_exter_login_request, request_list, secret_view,
    stat, sub_app_list, sub_app_notify_get_config, sub_app_notify_set_config, sub_app_request,
    sub_app_secret_view, sub_request_list,
};
#[post("/{method}")]
pub(crate) async fn base(
    bearer: BearerQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => mapping_data(&req_query, &auth_dao, web_dao.as_ref()).await,
        "parent_app" => {
            parent_app_list(&json_param.param::<UserParentAppListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "stat" => stat(&json_param.param::<AppStatParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "add" => add(&json_param.param::<AddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "confirm" => confirm(&json_param.param::<ConfirmParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "change" => change(&json_param.param::<ChangeParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "app_secret_add" => {
            app_secret_add(&json_param.param::<AddAppSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "app_secret_change" => {
            app_secret_change(&json_param.param::<ChangeAppSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "app_secret_del" => {
            app_secret_del(&json_param.param::<DelAppSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "notify_secret_change" => {
            notify_secret_change(&json_param.param::<ChangeNotifySecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "secret_view" => {
            secret_view(&json_param.param::<SecretViewSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "sub_app_secret_view" => {
            sub_app_secret_view(&json_param.param::<SecretViewSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "list" => app_list(&json_param.param::<UserAppListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "request_list" => request_list(&json_param.param::<RequestListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "sub_request_list" => {
            sub_request_list(&json_param.param::<SubRequestListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "delete" => delete(&json_param.param::<DeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "request_exter_feature" => {
            request_exter_feature(&json_param.param::<RequestExterFeatureParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "confirm_exter_feature" => {
            confirm_exter_feature(&json_param.param::<ConfirmExterFeatureParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "request_inner_feature_exter_login_request" => {
            request_inner_feature_exter_login_request(
                &json_param.param::<RequestExterLoginFeatureParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "sub_app_request" => {
            sub_app_request(&json_param.param::<RequestExterSubAppParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "sub_app_list" => sub_app_list(&json_param.param::<SubAppListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "sub_app_notify_get_config" => {
            sub_app_notify_get_config(
                &json_param.param::<SubAppNotifyGetConfigParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "sub_app_notify_set_config" => {
            sub_app_notify_set_config(
                &json_param.param::<SubAppNotifySetConfigParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "oauth_client_scope_data" => {
            oauth_client_scope_data(&json_param.param::<OAuthClientScopeDataParam>()?, web_dao.as_ref())
                .await
        }
        "oauth_client_request" => {
            oauth_client_request(&json_param.param::<OAuthClientRequestParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oauth_client_scope_request" => {
            oauth_client_scope_request(&json_param.param::<OAuthClientRequestParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "oauth_client_set_domain" => {
            oauth_client_set_domain(
                &json_param.param::<ConfirmOAuthClientSetDomainParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "oauth_client_secret_add" => {
            oauth_secret_add(&json_param.param::<AddOAuthSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oauth_client_secret_change" => {
            oauth_secret_change(&json_param.param::<ChangeOAuthSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oauth_client_secret_del" => {
            oauth_secret_del(&json_param.param::<DelOAuthSecretParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oauth_server_client_confirm" => {
            oauth_server_client_confirm(&json_param.param::<ConfirmOAuthClientParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "oauth_server_client_scope_confirm" => {
            oauth_server_client_scope_confirm(
                &json_param.param::<ConfirmOAuthClientScopeParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "oauth_server_request" => {
            oauth_server_request(&json_param.param::<OAuthServerRequestParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oauth_server_setting" => {
            oauth_server_setting(
                &json_param.param::<ConfirmOAuthServerSettingParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "notify_list" => {
            notify_data_list(&json_param.param::<NotifyDataListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "notify_del" => {
            notify_data_del(&json_param.param::<NotifyDataDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
