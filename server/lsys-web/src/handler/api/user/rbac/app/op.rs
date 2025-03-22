use crate::common::{JsonResult, PageParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{OpDataParam, RbacOpAddData, RbacOpData};
use lsys_rbac::model::RbacOpModel;

//用户后台对APP的RBAC操作管理
pub struct AppOpAddParam {
    pub app_id: u64,
    pub op_key: String,
    pub op_name: String,
    pub data: String,
}
pub async fn app_op_add(param: &AppOpAddParam, req_dao: &UserAuthQueryDao) -> JsonResult<u64> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .add_op(
            &RbacOpAddData {
                user_id: auth_data.user_id(),
                app_id: Some(0),
                op_info: RbacOpData {
                    op_key: &param.op_key,
                    op_name: if param.op_name.is_empty() {
                        None
                    } else {
                        Some(&param.op_name)
                    },
                },
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(id)
}

pub struct AppOpEditParam {
    pub app_id: u64,
    pub op_id: u64,
    pub op_key: String,
    pub op_name: String,
    pub data: String,
}

pub async fn app_op_edit(
    param: &AppOpEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .edit_op(
            &op,
            &RbacOpData {
                op_key: &param.op_key,
                op_name: if param.op_name.is_empty() {
                    None
                } else {
                    Some(&param.op_name)
                },
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(())
}
pub struct AppOpDelData {
    pub app_id: u64,
    pub op_id: u64,
}

pub async fn app_op_del(param: &AppOpDelData, req_dao: &UserAuthQueryDao) -> JsonResult<()> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .del_op(&op, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(())
}

pub struct AppOpDataParam {
    pub app_id: u64,
    pub op_name: Option<String>,
    pub op_key: Option<String>,
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn app_op_data(
    param: &AppOpDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<RbacOpModel>, Option<i64>)> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .op_data(
            &OpDataParam {
                user_id: auth_data.user_id(),
                app_id: Some(0),
                op_name: param.op_name.as_deref(),
                op_key: param.op_key.as_deref(),
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
                .op
                .op_count(&OpDataParam {
                    user_id: auth_data.user_id(),
                    app_id: Some(0),
                    op_name: param.op_name.as_deref(),
                    op_key: param.op_key.as_deref(),
                    ids: param.ids.as_deref(),
                })
                .await?,
        )
    } else {
        None
    };
    Ok((res, count))
}
