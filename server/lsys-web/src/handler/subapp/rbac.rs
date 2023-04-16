use lsys_app::model::AppsModel;
use lsys_rbac::dao::{AccessRes, RoleRelationKey};
use serde::Deserialize;

use crate::dao::WebDao;

use crate::handler::access::AccessSubAppRbacCheck;
use crate::handler::common::rbac::{
    rbac_access_check, rbac_menu_check, RbacAccessParam, RbacMenuParam, RelationParam,
};
use crate::{JsonData, JsonResult};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct CheckResParam {
    pub res: String,                     //资源KEY
    pub user_id: u64,                    //资源用户ID
    pub ops: Vec<String>,                //授权列表
    pub option_ops: Option<Vec<String>>, //可选授权列表
}

#[derive(Debug, Deserialize)]
pub struct CheckAccessParam {
    pub relation_key: Vec<RelationParam>,
    pub check_res: Vec<Vec<CheckResParam>>,
}

#[derive(Debug, Deserialize)]
pub struct CheckParam {
    pub user_id: u64,
    pub access: CheckAccessParam,
}

pub async fn subapp_rbac_check(
    app_dao: &WebDao,
    app: &AppsModel,
    param: CheckParam,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessSubAppRbacCheck {
                app: app.to_owned(),
            },
            None,
        )
        .await?;
    let dao = &app_dao.user.rbac_dao.rbac.access;
    let rkey = param
        .access
        .relation_key
        .into_iter()
        .map(|e| RoleRelationKey {
            relation_key: e.role_key,
            user_id: e.user_id,
        })
        .collect::<Vec<RoleRelationKey>>();
    let check_res = param
        .access
        .check_res
        .into_iter()
        .map(|e| {
            e.into_iter()
                .map(|p| AccessRes {
                    res: p.res,
                    user_id: p.user_id,
                    ops: p.ops,
                    option_ops: p.option_ops.unwrap_or_default(),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<Vec<_>>>();
    dao.list_check(param.user_id, &rkey, &check_res).await?;
    Ok(JsonData::data(json!({ "pass": 1 })))
}

#[derive(Debug, Deserialize)]
pub struct MenuParam {
    pub user_id: u64,
    pub menu: RbacMenuParam,
}

pub async fn subapp_rbac_menu_check(
    app_dao: &WebDao,
    app: &AppsModel,
    param: MenuParam,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessSubAppRbacCheck {
                app: app.to_owned(),
            },
            None,
        )
        .await?;
    rbac_menu_check(param.user_id, param.menu, &app_dao.user.rbac_dao).await
}

#[derive(Debug, Deserialize)]
pub struct AccessParam {
    pub user_id: u64,
    pub access: RbacAccessParam,
}

pub async fn subapp_rbac_access_check(
    app_dao: &WebDao,
    app: &AppsModel,
    param: AccessParam,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessSubAppRbacCheck {
                app: app.to_owned(),
            },
            None,
        )
        .await?;
    rbac_access_check(param.user_id, param.access, &app_dao.user.rbac_dao).await
}
