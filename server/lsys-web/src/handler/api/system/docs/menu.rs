use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::{common::JsonResponse, dao::access::api::system::CheckAdminDocs};
use lsys_access::dao::AccessSession;
use lsys_docs::dao::GitDocsMenuData;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct MenuAddParam {
    pub tag_id: u64,
    pub menu_path: String,
}

pub async fn menu_add(
    param: &MenuAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
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
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct MenuDelParam {
    pub menu_id: u64,
}

pub async fn menu_del(
    param: &MenuDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminDocs {})
        .await?;

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
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct MenuListParam {
    pub tag_id: u64,
}

pub async fn menu_list(
    param: &MenuListParam,
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
    let data = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .menu_list(&tag)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "data": data }))))
}
