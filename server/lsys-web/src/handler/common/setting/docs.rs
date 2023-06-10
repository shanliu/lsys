use lsys_docs::dao::{DocFile, GitDocResult, GitDocsGitAdd, GitDocsGitEdit, GitDocsMenuItem};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;
use tokio::fs::read;

use crate::{
    dao::{RequestDao, WebDao},
    handler::access::AccessAdminDocsEdit,
    JsonData, JsonResult, PageParam,
};

pub async fn docs_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminDocsEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let data = req_dao.web_dao.docs.docs.list_data().await?;
    Ok(JsonData::data(json!({
        "data":data
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsLogsParam {
    pub git_id: u32,
    pub host: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn docs_logs<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsLogsParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminDocsEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let data = req_dao
        .web_dao
        .docs
        .docs
        .logs_data(
            &param.git_id,
            &param.host,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .docs
                .docs
                .logs_count(&param.git_id, &param.host)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({
        "data":data,
        "total":count,
    })))
}
#[derive(Debug, Deserialize)]
pub struct DocsMenuParam {
    pub menu_path: String,
    pub access_path: Option<String>,
}
#[derive(Debug, Deserialize)]
pub struct DocsAddParam {
    pub url: String,
    pub branch: String,
    pub build_version: String,
    pub is_update: bool,
    pub is_tag: bool,
    pub menu_data: Vec<DocsMenuParam>,
    pub clear_rule: Option<Vec<String>>,
}

pub async fn docs_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminDocsEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;

    let id = req_dao
        .web_dao
        .docs
        .docs
        .add_git(
            &GitDocsGitAdd {
                url: param.url,
                branch: param.branch,
                build_version: param.build_version,
                is_update: param.is_update,
                is_tag: param.is_tag,
                menu_data: param
                    .menu_data
                    .into_iter()
                    .map(|e| GitDocsMenuItem {
                        menu_path: e.menu_path,
                        access_path: e.access_path,
                    })
                    .collect::<Vec<_>>(),
                clear_rule: param.clear_rule,
            },
            req_auth.user_data().user_id,
        )
        .await?;
    let _ = req_dao.web_dao.docs.task.notify();

    Ok(JsonData::data(json!({
        "id":id,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsEditParam {
    pub doc_id: u32,
    pub branch: String,
    pub build_version: String,
    pub is_update: bool,
    pub is_tag: bool,
    pub menu_data: Vec<DocsMenuParam>,
    pub clear_rule: Option<Vec<String>>,
}

pub async fn docs_edit<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsEditParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminDocsEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;
    let doc = req_dao.web_dao.docs.docs.find_by_id(&param.doc_id).await?;
    req_dao
        .web_dao
        .docs
        .docs
        .edit_git(
            &doc,
            &GitDocsGitEdit {
                branch: param.branch,
                build_version: param.build_version,
                is_update: param.is_update,
                is_tag: param.is_tag,
                menu_data: param
                    .menu_data
                    .into_iter()
                    .map(|e| GitDocsMenuItem {
                        menu_path: e.menu_path,
                        access_path: e.access_path,
                    })
                    .collect::<Vec<_>>(),
                clear_rule: param.clear_rule,
            },
            req_auth.user_data().user_id,
        )
        .await?;
    let _ = req_dao.web_dao.docs.task.notify();
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsGitDetailParam {
    pub url: String,
}

pub async fn docs_git_detail<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsGitDetailParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    //验证权限
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminDocsEdit {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await?;

    let data = req_dao.web_dao.docs.docs.git_detail(param.url).await?;

    Ok(JsonData::data(json!({
        "data":data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsRawReadParam {
    pub menu_id: u64,
    pub url: String,
}

pub async fn docs_file(param: &DocsRawReadParam, webdao: &WebDao) -> GitDocResult<DocFile> {
    webdao.docs.docs.open_file(param.menu_id, &param.url).await
}

pub async fn docs_menu(webdao: &WebDao) -> JsonResult<JsonData> {
    let data = webdao.docs.docs.menu().await?;
    Ok(JsonData::data(json!({
        "data":data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsMdReadParam {
    pub url: String,
    pub menu_id: u64,
}

pub async fn docs_md_read(param: DocsMdReadParam, webdao: &WebDao) -> JsonResult<JsonData> {
    let data = webdao
        .docs
        .docs
        .open_file(param.menu_id, &param.url)
        .await?;
    let dat = read(data.path).await?;
    Ok(JsonData::data(json!({
        "id":data.build_id,
        "version": data.version,
        "data":String::from_utf8_lossy(&dat),
    })))
}
