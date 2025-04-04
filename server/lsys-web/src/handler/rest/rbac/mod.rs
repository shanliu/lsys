mod check;
mod op;
mod res;
mod res_type;
mod role;
mod role_perm;
mod role_user;
pub use check::*;
use lsys_app::model::AppModel;
use lsys_core::fluent_message;
pub use op::*;
pub use res::*;
pub use res_type::*;
pub use role::*;
pub use role_perm::*;
pub use role_user::*;

use crate::{
    common::{JsonData, JsonError, JsonResult, RequestDao},
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
        .check(&req_dao.req_env, None, &CheckRestApp {})
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(app, &[crate::handler::APP_FEATURE_RBAC])
        .await?;

    Ok(())
}
//校验APP是否相同
fn inner_app_self_check(app: &AppModel, res_app_id: u64) -> JsonResult<()> {
    if app.id != res_app_id {
        return Err(JsonError::JsonData(
            JsonData::default().set_code(403),
            fluent_message!("bad_app_id"),
        ));
    }
    Ok(())
}

async fn inner_user_data_to_user_id(
    app: &AppModel,
    user_data: Option<&str>,
    req_dao: &RequestDao,
) -> JsonResult<u64> {
    let target_user_id = match user_data.and_then(|e| {
        if e.trim_matches(['\n', ' ', '\t']).is_empty() {
            None
        } else {
            Some(e)
        }
    }) {
        Some(user_data) => {
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .cache()
                .sync_user(app.id, user_data, None, None)
                .await?
                .id
        }
        None => app.user_id,
    };
    Ok(target_user_id)
}
