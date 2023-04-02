use lsys_app::model::AppsModel;
use serde::Deserialize;

use crate::dao::WebDao;

use crate::handler::access::AccessAppRbacCheck;
use crate::handler::common::rbac::{
    rbac_access_check, rbac_menu_check, AccessCheckParam, RbacMenuParam,
};
use crate::{JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct CheckParam {
    pub user_id: u64,
    pub access: AccessCheckParam,
}

pub async fn subapp_rbac_access_check(
    app_dao: &WebDao,
    app: &AppsModel,
    param: CheckParam,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessAppRbacCheck {
            app: app.to_owned(),
        })
        .await?;
    rbac_access_check(param.user_id, param.access, &app_dao.user.rbac_dao).await
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
        .check(&AccessAppRbacCheck {
            app: app.to_owned(),
        })
        .await?;
    rbac_menu_check(param.user_id, param.menu, &app_dao.user.rbac_dao).await
}
