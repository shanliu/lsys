use lsys_access::dao::AccessSession;
use lsys_docs::dao::GitDocsMenuData;
use serde::Deserialize;
use serde_json::json;

use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::{common::JsonData, dao::access::api::system::CheckAdminDocs};

#[derive(Debug, Deserialize)]
pub struct DocsMenuAddParam {
    pub tag_id: u64,
    pub menu_path: String,
}

pub async fn docs_menu_add(
    param: &DocsMenuAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?; //验证权限

    let tag_m = req_dao
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
        .menu_add(
            &tag_m,
            &GitDocsMenuData {
                menu_path: &param.menu_path,
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsMenuDelParam {
    pub menu_id: u64,
}

pub async fn docs_menu_del(
    param: &DocsMenuDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?; //验证权限

    let menu = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .find_menu_by_id(&param.menu_id)
        .await?;
    req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .menu_del(&menu, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct DocsMenuListParam {
    pub tag_id: u64,
}

pub async fn docs_menu_list(
    param: &DocsMenuListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminDocs {}, None)
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
        .menu_list(&tag)
        .await?;
    Ok(JsonData::data(json!({ "data": data })))
}
