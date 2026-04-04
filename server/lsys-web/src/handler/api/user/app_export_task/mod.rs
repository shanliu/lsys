mod active_count;
mod list;
mod mapping;
mod submit;
pub use active_count::*;
pub use list::*;
pub use mapping::*;
pub use submit::*;

use crate::{
    common::{JsonResult, UserAuthQueryDao},
    dao::access::{
        api::system::user::{CheckUserAppEdit, CheckUserAppView},
        RbacAccessCheckEnv,
    },
};
use lsys_app::model::AppModel;
use lsys_user::dao::UserAuthData;

/// 校验用户对指定 app 的权限，并返回 AppModel
///
/// 参考 rbac/app/mod.rs 中的 app_check_get 实现
async fn app_check_get(
    app_id: u64,
    is_edit: bool,
    auth_data: &UserAuthData,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<AppModel> {
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(app_id)
        .await?;

    if is_edit {
        req_dao
            .web_dao
            .web_rbac
            .check(
                &RbacAccessCheckEnv::session_body(auth_data, &req_dao.req_env),
                &CheckUserAppEdit {
                    res_user_id: app.user_id,
                },
            )
            .await?;
    } else {
        req_dao
            .web_dao
            .web_rbac
            .check(
                &RbacAccessCheckEnv::session_body(auth_data, &req_dao.req_env),
                &CheckUserAppView {
                    res_user_id: app.user_id,
                },
            )
            .await?;
    }
    app.app_status_check()?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_FILE])
        .await?;
    Ok(app)
}
