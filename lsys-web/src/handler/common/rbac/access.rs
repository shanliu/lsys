use lsys_rbac::dao::{AccessRes, AccessResOp, MenuAccess, MenuResult, RbacDao, RoleRelationKey};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{JsonData, JsonResult, RelationParam};

#[derive(Debug, Deserialize)]
pub struct CheckOpParam {
    pub name: String,
    pub authorize: bool,
}

#[derive(Debug, Deserialize)]
pub struct CheckResParam {
    pub res: String,            //资源KEY
    pub user_id: u64,           //资源用户ID
    pub ops: Vec<CheckOpParam>, //授权列表
}

#[derive(Debug, Deserialize)]
pub struct AccessCheckParam {
    pub relation_key: Vec<RelationParam>,
    pub check_res: Vec<Vec<CheckResParam>>,
}

impl From<CheckResParam> for AccessRes {
    fn from(p: CheckResParam) -> AccessRes {
        AccessRes {
            res: p.res,
            user_id: p.user_id,
            ops: p
                .ops
                .into_iter()
                .map(|e| access_op!(e.name, e.authorize))
                .collect::<Vec<AccessResOp>>(),
        }
    }
}

pub async fn rbac_access_check(
    user_id: u64,
    param: AccessCheckParam,
    rbac_dao: &RbacDao,
) -> JsonResult<JsonData> {
    let dao = &rbac_dao.rbac.access;
    let rkey = param
        .relation_key
        .into_iter()
        .map(|e| e.into())
        .collect::<Vec<RoleRelationKey>>();
    let check_res = param
        .check_res
        .into_iter()
        .map(|e| {
            e.into_iter()
                .map(AccessRes::from)
                .collect::<Vec<AccessRes>>()
        })
        .collect::<Vec<Vec<AccessRes>>>();
    dao.check(user_id, &rkey, &check_res).await?;
    Ok(JsonData::message("set name succ").set_data(json!({ "pass": 1 })))
}

#[derive(Debug, Deserialize)]
pub struct MenuCheckItemParam {
    pub name: String,
    pub access_check: Vec<Vec<CheckResParam>>,
}

#[derive(Debug, Deserialize)]
pub struct MenuCheckParam {
    pub relation_key: Vec<RelationParam>,
    pub check_res: Vec<MenuCheckItemParam>,
}

#[derive(Debug, Serialize)]
pub struct MenuData {
    pub status: bool, //是否授权成功
    pub name: String, //菜单名或key,参见:MenuAccess.name
}

impl From<MenuResult> for MenuData {
    fn from(p: MenuResult) -> MenuData {
        MenuData {
            status: p.result.map(|_| true).unwrap_or(false),
            name: p.name,
        }
    }
}

pub async fn rbac_menu_check(
    user_id: u64,
    param: MenuCheckParam,
    rbac_dao: &RbacDao,
) -> JsonResult<JsonData> {
    let dao = &rbac_dao.rbac.access;
    let rkey = param
        .relation_key
        .into_iter()
        .map(|e| e.into())
        .collect::<Vec<RoleRelationKey>>();
    let check_res = param
        .check_res
        .into_iter()
        .map(|e| {
            let access_res = e
                .access_check
                .into_iter()
                .map(|e| {
                    e.into_iter()
                        .map(AccessRes::from)
                        .collect::<Vec<AccessRes>>()
                })
                .collect::<Vec<Vec<AccessRes>>>();
            MenuAccess {
                access_res,
                name: e.name,
            }
        })
        .collect::<Vec<MenuAccess>>();
    let data = dao
        .menu_check(user_id, &rkey, &check_res)
        .await
        .into_iter()
        .map(MenuData::from)
        .collect::<Vec<MenuData>>();
    Ok(JsonData::message("check menu record").set_data(json!({ "data": data })))
}
