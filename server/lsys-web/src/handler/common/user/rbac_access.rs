use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::Value;

use crate::handler::common::rbac::{
    rbac_access_check, rbac_menu_check, AccessCheckParam, CheckResParam, RbacMenuItemParam,
    RbacMenuParam,
};
use crate::{dao::RequestDao, JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct UserAccessCheckParam {
    pub check_res: Vec<Vec<CheckResParam>>,
}

pub async fn user_access_check<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: UserAccessCheckParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let uids = param
        .check_res
        .iter()
        .flat_map(|n| n.iter().map(|c| c.user_id).collect::<Vec<u64>>())
        .collect::<Vec<u64>>();
    let relation_key = req_dao.get_user_relation_role(&uids).await;
    rbac_access_check(
        req_auth.user_data().user_id,
        AccessCheckParam {
            relation_key,
            check_res: param.check_res,
        },
        &req_dao.web_dao.user.rbac_dao,
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct UserMenuParam {
    pub check_res: Vec<UserMenuItemParam>,
}

#[derive(Debug, Deserialize)]
pub struct UserMenuItemParam {
    pub name: String,
    pub data: Value,
}

pub async fn user_menu_check<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: UserMenuParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let uids = param
        .check_res
        .iter()
        .filter_map(|e| match e.name.as_str() {
            "res-view" | "res-edit" => e.data.as_u64(),
            _ => None,
        })
        .collect::<Vec<_>>();
    let relation = req_dao.get_user_relation_role(&uids).await;
    rbac_menu_check(
        req_auth.user_data().user_id,
        RbacMenuParam {
            check_res: param
                .check_res
                .into_iter()
                .map(|e| RbacMenuItemParam {
                    name: e.name,
                    data: e.data,
                    relation: relation.clone(),
                })
                .collect::<Vec<_>>(),
        },
        &req_dao.web_dao.user.rbac_dao,
    )
    .await
}
