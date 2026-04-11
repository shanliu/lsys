use crate::{
    common::{
        JsonData, JsonError, JsonResponse, JsonResult, PageParam, ToOffsetPageParam,
        UserAuthQueryDao,
    },
    dao::access::{RbacAccessCheckEnv, api::system::admin::CheckAdminApp},
};
use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use lsys_core::fluents::IntoFluentMessage;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ConfirmExterFeatureParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_req_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub confirm_status: i8,
    pub confirm_note: String,
}
// APP功能审核,如邮件,短信等
pub async fn confirm_exter_feature(
    param: &ConfirmExterFeatureParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminApp {},
        )
        .await?;
    let req_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .request_find_by_id(param.app_req_id)
        .await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(req_app.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .exter_feature_confirm(
            &app,
            &req_app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ExterFeatureAddParam {
    pub feature_key: String,
    pub title: String,
}

pub async fn exter_feature_add(
    param: &ExterFeatureAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminApp {},
        )
        .await?;

    let id = req_dao
        .web_dao
        .web_app
        .exter_feature_add(
            &param.feature_key,
            &crate::dao::WebExterFeatureSetting {
                title: param.title.clone(),
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| JsonError::Message(e.to_fluent_message()))?;

    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct ExterFeatureEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub feature_key: String,
    pub title: String,
}

pub async fn exter_feature_edit(
    param: &ExterFeatureEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminApp {},
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .exter_feature_edit(
            param.id,
            &param.feature_key,
            &crate::dao::WebExterFeatureSetting {
                title: param.title.clone(),
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| JsonError::Message(e.to_fluent_message()))?;

    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ExterFeatureDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

pub async fn exter_feature_del(
    param: &ExterFeatureDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminApp {},
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .exter_feature_del(param.id, auth_data.user_id(), Some(&req_dao.req_env))
        .await
        .map_err(|e| JsonError::Message(e.to_fluent_message()))?;

    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ExterFeatureListParam {
    pub page: Option<PageParam>,
}

pub async fn exter_feature_list(
    param: &ExterFeatureListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminApp {},
        )
        .await?;

    let list = req_dao
        .web_dao
        .web_app
        .exter_feature_list(&param.page.to_offset_page_param())
        .await?;

    let data = list
        .into_iter()
        .map(|item| {
            json!({
                "id": item.id,
                "key": item.key,
                "title": item.data.title,
            })
        })
        .collect::<Vec<_>>();

    Ok(JsonResponse::data(JsonData::body(json!({ "data": data }))))
}
