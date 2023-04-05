use lsys_rbac::dao::RbacDao;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{JsonData, JsonResult};

use super::{access_check, RbacAccessParam};

pub async fn rbac_access_check(
    user_id: u64,
    param: RbacAccessParam,
    rbac_dao: &RbacDao,
) -> JsonResult<JsonData> {
    access_check(rbac_dao, user_id, &param).await?;
    Ok(JsonData::message("success").set_data(json!({ "pass": 1 })))
}

#[derive(Debug, Deserialize)]
pub struct RbacMenuParam {
    pub check_res: Vec<RbacAccessParam>,
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
        out.push(RbacMenuStatus {
            status: access_check(rbac_dao, user_id, &e)
                .await
                .map(|_| true)
                .unwrap_or(false),
            name: e.name,
        })
    }
    Ok(JsonData::message("check menu record").set_data(json!({ "data": out })))
}
