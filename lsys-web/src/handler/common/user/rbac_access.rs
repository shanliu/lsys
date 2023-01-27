use std::collections::HashSet;

use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;

use crate::handler::common::rbac::{
    rbac_access_check, rbac_menu_check, AccessCheckParam, CheckOpParam, CheckResParam,
    MenuCheckItemParam, MenuCheckParam,
};
use crate::{dao::RequestDao, JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct ResParam {
    pub res: String,      //资源KEY
    pub user_id: u64,     //资源用户ID
    pub ops: Vec<String>, //授权列表
}

#[derive(Debug, Deserialize)]
pub struct AccessParam {
    pub check_res: Vec<Vec<ResParam>>,
}

pub async fn user_access_check<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AccessParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let uids = param
        .check_res
        .iter()
        .flat_map(|n| n.iter().map(|c| c.user_id).collect::<Vec<u64>>())
        .collect::<Vec<u64>>();
    let relation_key = req_dao.get_user_relation_role(&uids).await;
    let check_res = param
        .check_res
        .into_iter()
        .map(|t| {
            t.into_iter()
                .map(|s| {
                    let ops = s
                        .ops
                        .into_iter()
                        .map(|name| CheckOpParam {
                            name,
                            authorize: true,
                        })
                        .collect::<Vec<CheckOpParam>>();
                    CheckResParam {
                        res: s.res,
                        user_id: s.user_id,
                        ops,
                    }
                })
                .collect::<Vec<CheckResParam>>()
        })
        .collect();
    rbac_access_check(
        req_auth.user_data().user_id,
        AccessCheckParam {
            relation_key,
            check_res,
        },
        &req_dao.web_dao.user.rbac_dao,
    )
    .await
}

#[derive(Debug, Deserialize)]
pub struct MenuItemParam {
    pub name: String,
    pub access_check: Vec<Vec<ResParam>>,
}

#[derive(Debug, Deserialize)]
pub struct MenuParam {
    pub check_res: Vec<MenuItemParam>,
}

pub async fn user_menu_check<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MenuParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let uids = param
        .check_res
        .iter()
        .flat_map(|e| {
            e.access_check
                .iter()
                .flat_map(|n| n.iter().map(|c| c.user_id).collect::<Vec<u64>>())
                .collect::<Vec<u64>>()
        })
        .collect::<HashSet<u64>>()
        .into_iter()
        .collect::<Vec<u64>>();
    let relation_key = req_dao.get_user_relation_role(&uids).await;
    let check_res = param
        .check_res
        .into_iter()
        .map(|e| {
            let access_check = e
                .access_check
                .into_iter()
                .map(|t| {
                    t.into_iter()
                        .map(|s| {
                            let ops = s
                                .ops
                                .into_iter()
                                .map(|name| CheckOpParam {
                                    name,
                                    authorize: true,
                                })
                                .collect::<Vec<CheckOpParam>>();
                            CheckResParam {
                                res: s.res,
                                user_id: s.user_id,
                                ops,
                            }
                        })
                        .collect::<Vec<CheckResParam>>()
                })
                .collect::<Vec<Vec<CheckResParam>>>();
            MenuCheckItemParam {
                name: e.name,
                access_check,
            }
        })
        .collect::<Vec<MenuCheckItemParam>>();
    rbac_menu_check(
        req_auth.user_data().user_id,
        MenuCheckParam {
            relation_key,
            check_res,
        },
        &req_dao.web_dao.user.rbac_dao,
    )
    .await
}
