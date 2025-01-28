use lsys_access::dao::AccessSession;
use lsys_docs::dao::GitDocsData;
use serde::Deserialize;
use serde_json::json;

use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::{common::JsonData, dao::access::api::system::CheckAdminDocs};

#[derive(Debug, Deserialize)]
pub struct DocsGitAddParam {
    pub name: String,
    pub url: String,
    pub max_try: u8,
}
pub async fn docs_git_add(
    param: &DocsGitAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?; //验证权限
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
pub struct DocsGitEditParam {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub max_try: u8,
}
pub async fn docs_git_edit(
    param: &DocsGitEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
        .await?;

    let auth_data = req_dao.user_session.read().await.get_session_data().await?; //验证权限
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
pub struct DocsGitDelParam {
    pub id: u32,
    pub timeout: Option<u8>,
}
pub async fn docs_git_del(
    param: &DocsGitDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?; //验证权限

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

pub async fn docs_git_list(req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
        .await?;
    let data = req_dao.web_dao.web_doc.docs_dao.docs.git_list().await?;
    Ok(JsonData::data(json!({ "data": data })))
}

#[derive(Debug, Deserialize)]
pub struct DocsGitDetailParam {
    pub url: String,
}

pub async fn docs_git_detail(
    param: &DocsGitDetailParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
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
