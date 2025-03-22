mod check;
mod op;
mod res;
mod role;
pub use check::*;
use lsys_app::model::AppModel;
pub use op::*;
pub use res::*;
pub use role::*;

use crate::{
    common::{JsonResult, RequestDao},
    dao::access::rest::CheckRestApp,
};

//当用户ID为APP应用的用户ID时,作为外部应用系统RBAC权限
//当用户ID为APP应用的子用户ID时,作为外部应用用户RBAC权限
//外部用户需要固定一个user_data作为系统用户标识

//校验APP是否开通RBAC功能
async fn inner_app_rbac_check(app: &AppModel, req_dao: &RequestDao) -> JsonResult<()> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, None, &CheckRestApp { app_id: app.id })
        .await?;
    req_dao.web_dao.web_rbac.app_feature_check(app).await?;
    Ok(())
}
