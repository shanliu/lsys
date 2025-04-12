use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::UserAuthQueryDao;
use crate::{common::JsonResponse, dao::access::api::system::CheckAdminDocs};
use lsys_access::dao::AccessSession;
use lsys_docs::{dao::GitDocsGitTag, model::DocGitTagStatus};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct TagAddParam {
    pub git_id: u32,
    pub tag: String,
    pub build_version: String,
    pub clear_rule: Option<Vec<String>>,
}

pub async fn tag_add(param: &TagAddParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
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
        .find_git_by_id(&param.git_id)
        .await?;
    let clear_rule = param
        .clear_rule
        .as_ref()
        .map(|e| e.iter().map(|e| e.as_str()).collect::<Vec<_>>());
    let id = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .tag_add(
            &git_m,
            &GitDocsGitTag {
                tag: &param.tag,
                build_version: &param.build_version,
                clear_rule: clear_rule.as_deref(),
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct TagDelParam {
    pub tag_id: u64,
    pub timeout: Option<u8>,
}

pub async fn tag_del(param: &TagDelParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

    let tag = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .find_tag_by_id(&param.tag_id)
        .await?;
    req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .tag_del(
            &tag,
            auth_data.user_id(),
            param.timeout.map(|e| e as u64).unwrap_or(60),
            Some(&req_dao.req_env),
        )
        .await?;
    let _ = req_dao.web_dao.web_doc.docs_dao.task.notify();
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct TagStatusParam {
    pub status: i8,
    pub tag_id: u64,
}
pub async fn tag_status(
    param: &TagStatusParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

    let tag = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .find_tag_by_id(&param.tag_id)
        .await?;
    let status = DocGitTagStatus::try_from(param.status)?;
    req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .tags_status(&tag, status, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct TagListParam {
    pub status: Option<i8>,
    pub key_word: Option<String>,
    pub git_id: Option<u32>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}

pub async fn tag_list(
    param: &TagListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

    let status = match param.status {
        Some(tmp) => Some(DocGitTagStatus::try_from(tmp)?),
        None => None,
    };
    let data = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .tags_list(
            param.git_id,
            status,
            param.key_word.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_doc
                .docs_dao
                .docs
                .tags_count(param.git_id, status, param.key_word.as_deref())
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": data,"total":count }),
    )))
}

#[derive(Debug, Deserialize)]
pub struct TagLogsParam {
    pub tag_id: u32,
}

pub async fn tag_logs(
    param: &TagLogsParam,
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
        .tags_logs(&param.tag_id)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "data": data }))))
}

#[derive(Debug, Deserialize)]
pub struct TagCLoneDelParam {
    pub clone_id: u64,
    pub timeout: Option<u8>,
}

pub async fn tag_clone_del(
    param: &TagCLoneDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

    let clone = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .find_clone_by_id(&param.clone_id)
        .await?;
    req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .tag_clone_del(
            &clone,
            param.timeout.map(|e| e as u64).unwrap_or(60),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct TagDirParam {
    pub tag_id: u64,
    pub prefix: Option<String>,
}
pub async fn tag_dir(param: &TagDirParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

    let tag = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .find_tag_by_id(&param.tag_id)
        .await?;
    let data = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .menu_file_list(&tag, &param.prefix.to_owned().unwrap_or_default())
        .await?
        .into_iter()
        .map(|e| {
            json!({
                "clone_id":e.clone_id,
                "url_path":e.url_path,
                "is_dir":e.is_dir,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":data,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct TagFileDataParam {
    pub tag_id: u64,
    pub file_path: String,
}
pub async fn tag_file_info(
    param: &TagFileDataParam,
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
        .docs_tag_file_info(param.tag_id, &param.file_path)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "id":data.id,
        "version": data.version,
        "data":data.data,
    }))))
}
