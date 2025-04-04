use crate::common::{JsonData, JsonResult, PageParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;
use lsys_rbac::{
    dao::{ResTypeListParam, ResTypeParam},
    model::RbacOpModel,
};
use serde::Deserialize;
use serde_json::json;

use super::{app_check_get, inner_user_data_to_user_id};

#[derive(Debug, Deserialize)]
pub struct AppResTypeListParam {
    pub app_id: u64,
    pub user_param: Option<String>,
    pub res_type: Option<String>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn app_res_type_data(
    param: &AppResTypeListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(&app, param.user_param.as_deref(), req_dao).await?;

    let res_param = ResTypeListParam {
        user_id: Some(user_id),
        app_id: Some(app.id),
        res_type: param
            .res_type
            .as_ref()
            .and_then(|e| {
                if e.trim_matches(['\n', ' ', '\t']).is_empty() {
                    None
                } else {
                    Some(e)
                }
            })
            .map(|e| e.as_str()),
    };

    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_data(&res_param, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_type_count(&res_param)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": rows,"total":count})))
}

#[derive(Debug, Deserialize)]
pub struct AppResTypeAddOpParam {
    pub app_id: u64,
    pub user_param: Option<String>,
    pub res_type: String,
    pub op_ids: Vec<u64>,
}

pub async fn app_res_type_op_add(
    param: &AppResTypeAddOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(&app, param.user_param.as_deref(), req_dao).await?;

    let op_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_ids(&param.op_ids)
        .await?
        .into_iter()
        .filter(|e| e.1.app_id == app.id)
        .map(|e| e.1)
        .collect::<Vec<RbacOpModel>>();
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_add_op(
            &ResTypeParam {
                res_type: &param.res_type,
                user_id,
                app_id: app.id,
            },
            &op_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct AppResDelOpParam {
    pub app_id: u64,
    pub user_param: Option<String>,
    pub res_type: String,
    pub op_ids: Vec<u64>,
}

pub async fn app_res_type_op_del(
    param: &AppResDelOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(&app, param.user_param.as_deref(), req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_del_op(
            &ResTypeParam {
                res_type: &param.res_type,
                user_id,
                app_id: app.id,
            },
            &param.op_ids,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct AppResTypeOpListParam {
    pub app_id: u64,
    pub user_param: Option<String>,
    pub res_type: String,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn app_res_type_op_data(
    param: &AppResTypeOpListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(&app, param.user_param.as_deref(), req_dao).await?;

    let res_param = ResTypeParam {
        res_type: &param.res_type,
        user_id,
        app_id: app.id,
    };

    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_op_data(
            &res_param,
            None,
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
                .res_type_op_count(&res_param)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": rows,"total":count})))
}
