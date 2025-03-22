use crate::{
    common::{JsonData, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::{CheckAdminRbacEdit, CheckAdminRbacView},
};
use lsys_access::dao::AccessSession;
use lsys_rbac::{
    dao::{ResTypeListParam, ResTypeParam},
    model::RbacOpModel,
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct AppResTypeListParam {
    pub res_type: String,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}
//@TODO 系统用户控制APP的资源
pub async fn rbac_res_type_data(
    param: &AppResTypeListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacView {})
        .await?;

    let res_param = ResTypeListParam {
        user_id: Some(auth_data.user_id()),
        app_id: Some(0),
        res_type: if param.res_type.is_empty() {
            None
        } else {
            Some(&param.res_type)
        },
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
    pub res_type: String,
    pub op_ids: Vec<u64>,
}

pub async fn app_res_op_add(
    param: &AppResTypeAddOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacEdit {})
        .await?;

    let op_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_ids(&param.op_ids)
        .await?
        .into_iter()
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
                user_id: auth_data.user_id(),
                app_id: 0,
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
    pub res_type: String,
    pub op_ids: Vec<u64>,
}

pub async fn app_res_type_op_del(
    param: &AppResDelOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacEdit {})
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_type_del_op(
            &ResTypeParam {
                res_type: &param.res_type,
                user_id: auth_data.user_id(),
                app_id: 0,
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
    pub res_type: String,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn app_res_type_op_data(
    param: &AppResTypeOpListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacView {})
        .await?;

    let res_param = ResTypeParam {
        res_type: &param.res_type,
        user_id: auth_data.user_id(),
        app_id: 0,
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
