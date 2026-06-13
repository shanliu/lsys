use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::system::app::{
    AppLogoutParam, ClearOAuthClientAccessTokenParam, ClearOAuthClientRefreshTokenParam,
    ConfirmExterFeatureParam, ConfirmExterLoginFeatureParam, ConfirmInnerFeatureSubAppParam,
    ConfirmOAuthClientParam, ConfirmOAuthClientScopeParam, ConfirmOAuthServerParam, ConfirmParam,
    DeleteParam, DisableParam, ExterFeatureAddParam, ExterFeatureDelParam, ExterFeatureEditParam,
    ExterFeatureListParam, ListParam, RequestListParam, SubListParam, app_list, app_logout,
    confirm, confirm_exter_feature, confirm_inner_feature_exter_login_confirm,
    confirm_inner_feature_sub_app_confirm, delete, disable, exter_feature_add, exter_feature_del,
    exter_feature_edit, exter_feature_list, mapping_data, oauth_client_clear_access_token,
    oauth_client_clear_refresh_token, oauth_client_confirm, oauth_client_scope_confirm,
    oauth_server_confirm, request_list, sub_app_list,
};

#[post("/{method}")]
pub(crate) async fn app(
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
        "confirm" => confirm(&json_param.param::<ConfirmParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "delete" => delete(&json_param.param::<DeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "auth_logout" => app_logout(&json_param.param::<AppLogoutParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "oauth_clear_access_token" => {
            oauth_client_clear_access_token(
                &json_param.param::<ClearOAuthClientAccessTokenParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "oauth_clear_refresh_token" => {
            oauth_client_clear_refresh_token(
                &json_param.param::<ClearOAuthClientRefreshTokenParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "disable" => disable(&json_param.param::<DisableParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "confirm_exter_feature" => {
            confirm_exter_feature(&json_param.param::<ConfirmExterFeatureParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "exter_feature_add" => {
            exter_feature_add(&json_param.param::<ExterFeatureAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "exter_feature_edit" => {
            exter_feature_edit(&json_param.param::<ExterFeatureEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "exter_feature_del" => {
            exter_feature_del(&json_param.param::<ExterFeatureDelParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "exter_feature_list" => {
            exter_feature_list(&json_param.param::<ExterFeatureListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "confirm_inner_feature_exter_login_confirm" => {
            confirm_inner_feature_exter_login_confirm(
                &json_param.param::<ConfirmExterLoginFeatureParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "confirm_inner_feature_sub_app_confirm" => {
            confirm_inner_feature_sub_app_confirm(
                &json_param.param::<ConfirmInnerFeatureSubAppParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "list" => app_list(&json_param.param::<ListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "sub_list" => sub_app_list(&json_param.param::<SubListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "mapping" => mapping_data(&req_query, web_dao.as_ref()).await,
        "request_list" => request_list(&json_param.param::<RequestListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await,
        "oauth_client_confirm" => {
            oauth_client_confirm(&json_param.param::<ConfirmOAuthClientParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oauth_client_scope_confirm" => {
            oauth_client_scope_confirm(
                &json_param.param::<ConfirmOAuthClientScopeParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "oauth_server_confirm" => {
            oauth_server_confirm(&json_param.param::<ConfirmOAuthServerParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
