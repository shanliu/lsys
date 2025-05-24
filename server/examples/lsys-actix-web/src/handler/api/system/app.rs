use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_web::post;
use lsys_web::handler::api::system::app::{
    app_list, confirm, confirm_exter_feature, confirm_inner_feature_exter_login_confirm,
    confirm_inner_feature_sub_app_confirm, delete, disable, mapping_data, oauth_client_confirm,
    oauth_client_scope_confirm, oauth_server_confirm, request_list, ConfirmExterFeatureParam,
    ConfirmExterLoginFeatureParam, ConfirmInnerFeatureSubAppParam, ConfirmOAuthClientParam,
    ConfirmOAuthClientScopeParam, ConfirmOAuthServerParam, ConfirmParam, DeleteParam, DisableParam,
    ListParam, RequestListParam,
};

#[post("/{method}")]
pub(crate) async fn app(
    jwt: JwtQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    auth_dao: UserAuthQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "confirm" => confirm(&json_param.param::<ConfirmParam>()?, &auth_dao).await,
        "delete" => delete(&json_param.param::<DeleteParam>()?, &auth_dao).await,
        "disable" => disable(&json_param.param::<DisableParam>()?, &auth_dao).await,
        "confirm_exter_feature" => {
            confirm_exter_feature(&json_param.param::<ConfirmExterFeatureParam>()?, &auth_dao).await
        }
        "confirm_inner_feature_exter_login_confirm" => {
            confirm_inner_feature_exter_login_confirm(
                &json_param.param::<ConfirmExterLoginFeatureParam>()?,
                &auth_dao,
            )
            .await
        }
        "confirm_inner_feature_sub_app_confirm" => {
            confirm_inner_feature_sub_app_confirm(
                &json_param.param::<ConfirmInnerFeatureSubAppParam>()?,
                &auth_dao,
            )
            .await
        }
        "list" => app_list(&json_param.param::<ListParam>()?, &auth_dao).await,
        "mapping" => mapping_data(&auth_dao).await,
        "request_list" => request_list(&json_param.param::<RequestListParam>()?, &auth_dao).await,
        "oauth_client_confirm" => {
            oauth_client_confirm(&json_param.param::<ConfirmOAuthClientParam>()?, &auth_dao).await
        }
        "oauth_client_scope_confirm" => {
            oauth_client_scope_confirm(
                &json_param.param::<ConfirmOAuthClientScopeParam>()?,
                &auth_dao,
            )
            .await
        }
        "oauth_server_confirm" => {
            oauth_server_confirm(&json_param.param::<ConfirmOAuthServerParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
