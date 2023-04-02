use lsys_rbac::dao::{AccessRes, RbacDao, RoleRelationKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    handler::access::{AccessAdminManage, AccessResEdit, AccessResView},
    JsonData, JsonResult, RelationParam,
};

#[derive(Debug, Deserialize)]
pub struct CheckResParam {
    pub res: String,                     //资源KEY
    pub user_id: u64,                    //资源用户ID
    pub ops: Vec<String>,                //授权列表
    pub option_ops: Option<Vec<String>>, //可选授权列表
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
            ops: p.ops,
            option_ops: p.option_ops.unwrap_or_default(),
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
        .map(|e| e.into_iter().map(AccessRes::from).collect::<Vec<_>>())
        .collect::<Vec<Vec<_>>>();
    dao.list_check(user_id, &rkey, &check_res).await?;
    Ok(JsonData::message("success").set_data(json!({ "pass": 1 })))
}

#[derive(Debug, Deserialize)]
pub struct RbacMenuParam {
    pub check_res: Vec<RbacMenuItemParam>,
}

#[derive(Deserialize, Debug)]
pub enum RbacMenuArgs {
    AccessResView { res_user_id: u64 },
}

#[derive(Debug, Deserialize)]
pub struct RbacMenuItemParam {
    pub name: String,
    pub data: Value,
    pub relation: Vec<RelationParam>,
}

#[derive(Debug, Serialize)]
pub struct RbacMenuStatus {
    pub status: bool, //是否授权成功
    pub name: String, //菜单名或key,参见:MenuItem.name
}

pub async fn rbac_menu_check(
    user_id: u64,
    param: RbacMenuParam,
    rbac_dao: &RbacDao,
) -> JsonResult<JsonData> {
    let mut out = Vec::with_capacity(param.check_res.len());
    for e in param.check_res.into_iter() {
        match e.name.as_str() {
            "admin-main" => out.push(RbacMenuStatus {
                name: e.name,
                status: rbac_dao
                    .rbac
                    .check(&AccessAdminManage { user_id })
                    .await
                    .map(|_| true)
                    .unwrap_or(false),
            }),
            "res-edit" => {
                let res_user_id = e.data.as_u64().unwrap_or(0);
                out.push(RbacMenuStatus {
                    name: e.name,
                    status: rbac_dao
                        .rbac
                        .check(&AccessResEdit {
                            user_id,
                            res_user_id,
                        })
                        .await
                        .map(|_| true)
                        .unwrap_or(false),
                });
            }
            "res-view" => {
                let res_user_id = e.data.as_u64().unwrap_or(0);
                out.push(RbacMenuStatus {
                    name: e.name,
                    status: rbac_dao
                        .rbac
                        .check(&AccessResView {
                            user_id,
                            res_user_id,
                        })
                        .await
                        .map(|_| true)
                        .unwrap_or(false),
                });
            }
            _ => {}
        }
    }
    Ok(JsonData::message("check menu record").set_data(json!({ "data": out })))
}
