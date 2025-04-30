use lsys_access::dao::AccessSession;
use lsys_docs::dao::GitDocsData;
use serde::Deserialize;
use serde_json::json;

use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::{common::JsonResponse, dao::access::api::system::CheckAdminDocs};

#[derive(Debug, Deserialize)]
pub struct GitAddParam {
    pub name: String,
    pub url: String,
    #[serde(deserialize_with = "crate::common::deserialize_u8")]
    pub max_try: u8,
}
pub async fn git_add(param: &GitAddParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;
    let id = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .git_add(
            &GitDocsData {
                name: &param.name,
                url: &param.url,
                max_try: param.max_try,
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct GitEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u32")]
    pub id: u32,
    pub name: String,
    pub url: String,
    #[serde(deserialize_with = "crate::common::deserialize_u8")]
    pub max_try: u8,
}
pub async fn git_edit(
    param: &GitEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

    let git_m = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .find_git_by_id(&param.id)
        .await?;
    req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .git_edit(
            &git_m,
            &GitDocsData {
                name: &param.name,
                url: &param.url,
                max_try: param.max_try,
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct GitDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u32")]
    pub id: u32,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u8")]
    pub timeout: Option<u8>,
}
pub async fn git_del(param: &GitDelParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

    let git_m = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .find_git_by_id(&param.id)
        .await?;
    req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .git_del(
            &git_m,
            auth_data.user_id(),
            param.timeout.map(|e| e as u64).unwrap_or(60),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

pub async fn git_list(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;
    let data = req_dao.web_dao.web_doc.docs_dao.docs.git_list().await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "data": data }))))
}

#[derive(Debug, Deserialize)]
pub struct GitDetailParam {
    pub url: String,
}

pub async fn git_detail(
    param: &GitDetailParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;
    let data = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .git_detail(&param.url)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":data,
    }))))
}
