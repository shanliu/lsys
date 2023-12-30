use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;

use crate::dao::RequestDao;

use crate::handler::access::{AccessUserAppEdit, AccessUserAppView};
use crate::{JsonData, JsonResult, PageParam};

#[derive(Debug, Deserialize)]
pub struct AppSetSubUserParam {
    pub app_id: u64,
    pub user_id: u64,
    pub used: bool,
}
//设置子用户
pub async fn app_set_sub_user<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppSetSubUserParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;

    req_dao
        .web_dao
        .app
        .app_dao
        .sub_app
        .set_sub_user(&app, &param.user_id, &param.used, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct AppListSubUserParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
//列出已设置的子用户
pub async fn app_list_sub_user<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppListSubUserParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppView {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;

    let out = req_dao
        .web_dao
        .app
        .app_dao
        .sub_app
        .list_sub_user_data(
            &app,
            &param.user_id,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        if param.user_id.is_some() {
            Some(out.len() as i64)
        } else {
            Some(
                req_dao
                    .web_dao
                    .app
                    .app_dao
                    .sub_app
                    .list_sub_user_count(&app)
                    .await?,
            )
        }
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": out,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct AppListSubAppParam {
    pub app_id: u64,
    pub user_id: Option<u64>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
//列出已注册子应用
pub async fn app_list_sub_app<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppListSubAppParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppView {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;

    let out = req_dao
        .web_dao
        .app
        .app_dao
        .sub_app
        .list_sub_app_data(
            &app,
            &param.user_id,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await?;

    let out = out
        .into_iter()
        .map(|e| {
            json!({
                 "app_id":e.sub_app.app_id,
                 "sub_app_id": e.sub_app.sub_app_id,
                 "sub_app_name": e.sub_app_name,
                 "sub_app_client_id": e.sub_app_client_id,
                 "sub_app_client_secret": e.sub_app.sub_client_secret,
                 "change_time":e.sub_app.change_time,
                 "status":e.sub_app.status,
                 "user_id":e.sub_app.user_id,
            })
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app
                .app_dao
                .sub_app
                .list_sub_app_count(&app, &param.user_id)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": out,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct ListParentAppParam {
    pub app_id: u64,
    pub is_set: Option<bool>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
//列出当前应用已关联的父应用
pub async fn list_parent_app<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ListParentAppParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppView {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;

    let out = req_dao
        .web_dao
        .app
        .app_dao
        .sub_app
        .parent_app_data(
            &app,
            &param.is_set,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await?;

    let out = out
        .into_iter()
        .map(|e| {
            json!({
                 "app_id":e.app_id,
                 "app_name":e.app_name,
                 "app_client_id":e.app_client_id,
                 "user_id":e.user_id,
                 "user_status":e.user_status,
                 "sub_app_name":e.sub_app.as_ref().map(|e|e.sub_app_name.as_str()),
                 "sub_app_client_id":e.sub_app.as_ref().map(|e|e.sub_app_client_id.as_str()),
                 "sub_app_status":e.sub_app.as_ref().map(|e|e.sub_app_status),
                 "sub_app_client_secret":e.sub_app.as_ref().map(|e|e.sub_app_client_secret.as_str()),
                 "change_time":e.sub_app.as_ref().map(|e|e.change_time),
            })
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app
                .app_dao
                .sub_app
                .parent_app_count(&app, &param.is_set)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": out,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct AppSetParentAppParam {
    pub app_id: u64,
    pub parent_app_id: u64,
    pub sub_secret: String,
}
//列出当前应用已关联的父应用
pub async fn app_set_parent_app<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppSetParentAppParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    let parent_app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.parent_app_id)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;

    req_dao
        .web_dao
        .app
        .app_dao
        .sub_app
        .app_parent_set(&parent_app, &app, &param.sub_secret, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct AppDelParentAppParam {
    pub app_id: u64,
    pub parent_app_id: u64,
}
//列出当前应用已关联的父应用
pub async fn app_del_parent_app<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppDelParentAppParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;

    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    let parent_app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.parent_app_id)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;

    req_dao
        .web_dao
        .app
        .app_dao
        .sub_app
        .app_parent_del(&parent_app, &app, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}
