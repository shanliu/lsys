use crate::{
    common::{JsonData, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::{CheckAdminRbacEdit, CheckAdminRbacView},
};
use lsys_access::dao::AccessSession;
use lsys_rbac::{dao::ResDataParam, model::RbacOpModel};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct RbacResAddParam {
    pub res_name: String,
    pub res_type: String,
    pub res_data: String,
}

pub async fn rbac_res_add(
    param: &RbacResAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminRbacEdit {}, None)
        .await?;

    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .add_res(
            0,
            &param.res_name,
            &param.res_type,
            &param.res_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct RbacResEditParam {
    pub res_id: u64,
    pub res_name: String,
    pub res_type: String,
    pub res_data: String,
}

pub async fn rbac_res_edit(
    param: &RbacResEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminRbacEdit {}, None)
        .await?;
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
            Some(&param.res_name),
            Some(&param.res_type),
            Some(&param.res_data),
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct RbacResDelParam {
    pub res_id: u64,
}

pub async fn rbac_res_del(
    param: &RbacResDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminRbacEdit {}, None)
        .await?;
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
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct RbacResParam {
    pub user_id: Option<u64>,
    pub res_type: Option<String>,
    pub res_data: Option<String>,
    pub res_name: Option<String>,
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn rbac_res_data(
    param: &RbacResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminRbacView {}, None)
        .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_data(
            &ResDataParam {
                user_id: param.user_id,
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
                    user_id: param.user_id,
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

    Ok(JsonData::data(json!({ "data": res,"total":count})))
}

#[derive(Debug, Deserialize)]
pub struct RbacResAddOpParam {
    pub res_id: u64,
    pub op_ids: Vec<u64>,
}

pub async fn rbac_res_op_add(
    param: &RbacResAddOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminRbacEdit {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
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
        .add_op(
            &res,
            &op_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct RbacResDelOpParam {
    pub res_id: u64,
    pub op_ids: Vec<u64>,
}

pub async fn rbac_res_op_del(
    param: &RbacResAddOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminRbacEdit {}, None)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .del_op(
            &res,
            &param.op_ids,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({ "data": rows})))
}

#[derive(Debug, Deserialize)]
pub struct RbacResListOpParam {
    pub res_id: u64,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn rbac_res_op_data(
    param: &RbacResListOpParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminRbacView {}, None)
        .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    let rows = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_op_data(&res, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_op_count(&res)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": rows,"total":count})))
}
