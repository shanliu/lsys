use lsys_app::model::AppModel;
use serde::Deserialize;

use crate::common::{JsonData, JsonResult, RequestDao};

use super::inner_app_rbac_check;

use crate::common::{PageParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;

use lsys_rbac::{
    dao::{RbacResAddData, RbacResData, ResDataParam},
    model::RbacResModel,
};

#[derive(Debug, Deserialize)]
pub struct UserResAddParam {
    user_data: String,
}

pub async fn user_res_add(
    param: &UserResAddParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    inner_app_rbac_check(app, req_dao).await?;
    //res.app_id=app.id
    //res.user_id=app.user_id
    let _res_user_id = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.user_data, None, None)
        .await?;
    todo!()
}

pub struct ResAddParam {
    pub res_name: String,
    pub res_type: String,
    pub res_data: String,
}

pub async fn res_add(
    param: &ResAddParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<u64> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .add_res(
            &RbacResAddData {
                user_id: auth_data.user_id(),
                app_id: Some(app.id),
                res_info: RbacResData {
                    res_name: if param.res_name.is_empty() {
                        None
                    } else {
                        Some(&param.res_name)
                    },
                    res_type: &param.res_type,
                    res_data: &param.res_data,
                },
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(id)
}

pub struct ResEditParam {
    pub res_id: u64,
    pub res_name: String,
    pub res_type: String,
    pub res_data: String,
}

pub async fn res_edit(
    param: &ResEditParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .edit_res(
            &op,
            &RbacResData {
                res_name: if param.res_name.is_empty() {
                    None
                } else {
                    Some(&param.res_name)
                },
                res_type: &param.res_type,
                res_data: &param.res_data,
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(())
}

pub struct ResDelParam {
    pub res_id: u64,
}

pub async fn res_del(
    param: &ResDelParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .del_res(&res, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(())
}

pub struct ResParam {
    pub res_type: Option<String>,
    pub res_data: Option<String>,
    pub res_name: Option<String>,
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn res_data(
    param: &ResParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<RbacResModel>, Option<i64>)> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_data(
            &ResDataParam {
                user_id: Some(auth_data.user_id()),
                app_id: Some(0),
                res_data: param.res_data.as_deref(),
                res_type: param.res_type.as_deref(),
                res_name: param.res_name.as_deref(),
                ids: param.ids.as_deref(),
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_count(&ResDataParam {
                    user_id: Some(auth_data.user_id()),
                    app_id: Some(0),
                    res_data: param.res_data.as_deref(),
                    res_type: param.res_type.as_deref(),
                    res_name: param.res_name.as_deref(),
                    ids: param.ids.as_deref(),
                })
                .await?,
        )
    } else {
        None
    };
    Ok((res, count))
}
