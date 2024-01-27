use lsys_docs::{
    dao::{DocPath, GitDocResult, GitDocsData, GitDocsGitTag, GitDocsMenuData},
    model::DocGitTagStatus,
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::fs::read;
use tracing::debug;

use crate::{
    dao::{RequestAuthDao, RequestDao, WebDao},
    handler::access::AccessAdminDocsEdit,
    JsonData, JsonResult, PageParam,
};

#[derive(Debug, Deserialize)]
pub struct DocsGitAddParam {
    pub name: String,
    pub url: String,
    pub max_try: u8,
}
pub async fn docs_git_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsGitAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let id = req_dao
        .web_dao
        .docs
        .docs
        .git_add(
            &GitDocsData {
                name: param.name,
                url: param.url,
                max_try: param.max_try,
            },
            req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct DocsGitEditParam {
    pub id: u32,
    pub name: String,
    pub url: String,
    pub max_try: u8,
}
pub async fn docs_git_edit<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsGitEditParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let git_m = req_dao
        .web_dao
        .docs
        .docs
        .find_git_by_id(&param.id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .docs
        .docs
        .git_edit(
            &git_m,
            &GitDocsData {
                name: param.name,
                url: param.url,
                max_try: param.max_try,
            },
            req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsGitDelParam {
    pub id: u32,
    pub timeout: Option<u8>,
}
pub async fn docs_git_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsGitDelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let git_m = req_dao
        .web_dao
        .docs
        .docs
        .find_git_by_id(&param.id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .docs
        .docs
        .git_del(
            &git_m,
            &req_auth.user_data().user_id,
            &param.timeout.map(|e| e as u64).unwrap_or(60),
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

pub async fn docs_git_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .docs
        .docs
        .git_list()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "data": data })))
}

#[derive(Debug, Deserialize)]
pub struct DocsTagAddParam {
    pub git_id: u32,
    pub tag: String,
    pub build_version: String,
    pub clear_rule: Option<Vec<String>>,
}

pub async fn docs_tag_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let git_m = req_dao
        .web_dao
        .docs
        .docs
        .find_git_by_id(&param.git_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let id = req_dao
        .web_dao
        .docs
        .docs
        .tag_add(
            &git_m,
            &GitDocsGitTag {
                tag: param.tag,
                build_version: param.build_version,
                clear_rule: param.clear_rule,
            },
            req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct DocsTagDelParam {
    pub tag_id: u64,
    pub timeout: Option<u8>,
}

pub async fn docs_tag_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagDelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tag = req_dao
        .web_dao
        .docs
        .docs
        .find_tag_by_id(&param.tag_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .docs
        .docs
        .tag_del(
            &tag,
            &req_auth.user_data().user_id,
            &param.timeout.map(|e| e as u64).unwrap_or(60),
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let _ = req_dao.web_dao.docs.task.notify();
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsTagListParam {
    pub status: Option<i8>,
    pub key_word: Option<String>,
    pub git_id: Option<u32>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
pub async fn docs_tag_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status = match param.status {
        Some(tmp) => Some(DocGitTagStatus::try_from(tmp).map_err(|e| req_dao.fluent_json_data(e))?),
        None => None,
    };
    let data = req_dao
        .web_dao
        .docs
        .docs
        .tags_list(
            &param.git_id,
            &status,
            &param.key_word,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .docs
                .docs
                .tags_count(&param.git_id, &status, &param.key_word)
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": data,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct DocsTagStatusParam {
    pub status: i8,
    pub tag_id: u64,
}
pub async fn docs_tag_status<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagStatusParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tag = req_dao
        .web_dao
        .docs
        .docs
        .find_tag_by_id(&param.tag_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status =
        DocGitTagStatus::try_from(param.status).map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .docs
        .docs
        .tags_status(
            &tag,
            &status,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsTagLogsParam {
    pub tag_id: u32,
}

pub async fn docs_tag_logs<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagLogsParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .docs
        .docs
        .tags_logs(&param.tag_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "data": data })))
}

#[derive(Debug, Deserialize)]
pub struct DocsTagCLoneDelParam {
    pub clone_id: u64,
    pub timeout: Option<u8>,
}

pub async fn docs_tag_clone_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagCLoneDelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let clone = req_dao
        .web_dao
        .docs
        .docs
        .find_clone_by_id(&param.clone_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .docs
        .docs
        .tag_clone_del(
            &clone,
            &param.timeout.map(|e| e as u64).unwrap_or(60),
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsGitDetailParam {
    pub url: String,
}

pub async fn docs_git_detail<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsGitDetailParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .docs
        .docs
        .git_detail(&param.url)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({
        "data":data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsTagDirParam {
    pub tag_id: u64,
    pub prefix: Option<String>,
}
pub async fn docs_tag_dir<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagDirParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tag = req_dao
        .web_dao
        .docs
        .docs
        .find_tag_by_id(&param.tag_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .docs
        .docs
        .menu_file_list(&tag, &param.prefix.unwrap_or_default())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?
        .into_iter()
        .map(|e| {
            json!({
                "clone_id":e.clone_id,
                "url_path":e.url_path,
                "is_dir":e.is_dir,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({
        "data":data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsTagFileDataParam {
    pub tag_id: u64,
    pub file_path: String,
}
pub async fn docs_tag_file_data<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsTagFileDataParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tag = req_dao
        .web_dao
        .docs
        .docs
        .find_tag_by_id(&param.tag_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .docs
        .docs
        .menu_file_read(&tag, &param.file_path)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let dat = read(data.file_path)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({
        "id":data.clone_id,
        "version": data.version,
        "data":String::from_utf8_lossy(&dat),
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsMenuAddParam {
    pub tag_id: u64,
    pub menu_path: String,
}

pub async fn docs_menu_add<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsMenuAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tag_m = req_dao
        .web_dao
        .docs
        .docs
        .find_tag_by_id(&param.tag_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .docs
        .docs
        .menu_add(
            &tag_m,
            &GitDocsMenuData {
                menu_path: param.menu_path,
            },
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsMenuDelParam {
    pub menu_id: u64,
}

pub async fn docs_menu_del<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsMenuDelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let menu = req_dao
        .web_dao
        .docs
        .docs
        .find_menu_by_id(&param.menu_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .docs
        .docs
        .menu_del(&menu, &req_auth.user_data().user_id, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsMenuListParam {
    pub tag_id: u64,
}

pub async fn docs_menu_list<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: DocsMenuListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?; //验证权限
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
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tag = req_dao
        .web_dao
        .docs
        .docs
        .find_tag_by_id(&param.tag_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .docs
        .docs
        .menu_list(&tag)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "data": data })))
}

#[derive(Debug, Deserialize)]
pub struct DocsRawReadParam {
    pub menu_id: u32,
    pub url: String,
}

pub async fn docs_file(param: &DocsRawReadParam, webdao: &WebDao) -> GitDocResult<DocPath> {
    webdao.docs.docs.menu_file(param.menu_id, &param.url).await
}

pub async fn docs_menu(req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .docs
        .docs
        .menu()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?
        .into_iter()
        .map(|e| {
            let mut err = None;
            let mut data = None;
            match e.menu_data {
                Ok(tmp) => match serde_json::from_slice::<Value>(&tmp) {
                    Ok(d) => data = Some(d),
                    Err(e) => err = Some(e.to_string()),
                },
                Err(e) => err = Some(e),
            }
            json!({
                "id":e.menu_id,
                "tag_id":e.tag_id,
                "version":e.version,
                "menu_path":e.menu_path,
                "menu_data":data,
                "menu_error":err
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({
        "data":data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsMdReadParam {
    pub url: String,
    pub menu_id: u32,
}
pub async fn docs_md_read(param: DocsMdReadParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .docs
        .docs
        .menu_file(param.menu_id, &param.url)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    debug!("read markdown file:{}", &data.file_path.to_string_lossy());
    let dat = read(data.file_path)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({
        "id":data.clone_id,
        "version": data.version,
        "data":String::from_utf8_lossy(&dat),
    })))
}
