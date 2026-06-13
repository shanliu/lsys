mod file_chunks;
mod file_delete;
mod file_download_progress;
mod file_from_url;
mod file_list;
mod file_logs;
mod file_operations;
mod file_upload;
mod mapping;
mod read;

pub use file_chunks::*;
pub use file_delete::*;
pub use file_download_progress::*;
pub use file_from_url::*;
pub use file_list::*;
pub use file_logs::*;
pub use file_operations::*;
pub use file_upload::*;
pub use mapping::*;
pub use read::*;

use crate::{
    common::{JsonResult, RequestDao},
    dao::{
        WebDao,
        access::{
            RbacAccessCheckEnv,
            api::system::user::{CheckUserAppEdit, CheckUserAppView},
        },
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
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<AppModel> {
    let app = web_dao.web_app.app_dao.app.find_by_id(app_id).await?;

    if is_edit {
        web_dao
            .web_rbac
            .check(
                &RbacAccessCheckEnv::session_body(auth_data, &req_dao.req_env),
                &CheckUserAppEdit {
                    res_user_id: app.user_id,
                },
            )
            .await?;
    } else {
        web_dao
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
    web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_FILE])
        .await?;
    Ok(app)
}
