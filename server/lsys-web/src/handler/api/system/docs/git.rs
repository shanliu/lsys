use lsys_access::dao::AccessSession;
use lsys_docs::dao::GitDocsData;
use serde::Deserialize;
use serde_json::json;

use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::{common::JsonData, dao::access::api::system::CheckAdminDocs};

#[derive(Debug, Deserialize)]
pub struct GitAddParam {
    pub name: String,
    pub url: String,
    pub max_try: u8,
}
pub async fn git_add(param: &GitAddParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
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
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct GitEditParam {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub max_try: u8,
}
pub async fn git_edit(param: &GitEditParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
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
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct GitDelParam {
    pub id: u32,
    pub timeout: Option<u8>,
}
pub async fn git_del(param: &GitDelParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
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
    Ok(JsonData::default())
}

pub async fn git_list(req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;
    let data = req_dao.web_dao.web_doc.docs_dao.docs.git_list().await?;
    Ok(JsonData::data(json!({ "data": data })))
}

#[derive(Debug, Deserialize)]
pub struct GitDetailParam {
    pub url: String,
}

pub async fn git_detail(
    param: &GitDetailParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
    Ok(JsonData::data(json!({
        "data":data,
    })))
}
