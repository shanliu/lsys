mod mapping;

mod record_file_list;
mod record_logs;
mod script_add;
mod script_data;
mod script_del;
mod script_detail;
mod script_edit;
mod script_files;
mod script_logs;
mod script_records;
mod script_status;
mod submit_task;

use lsys_app::model::AppModel;
use lsys_user::dao::UserAuthData;
pub use mapping::*;

pub use record_file_list::*;
pub use record_logs::*;
pub use script_add::*;
pub use script_data::*;
pub use script_del::*;
pub use script_detail::*;
pub use script_edit::*;
pub use script_files::*;
pub use script_logs::*;
pub use script_records::*;
pub use script_status::*;
pub use submit_task::*;

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

/// 校验用户对指定 app 的权限，并返回 AppModel
/// app_id 必须 > 0（用户应用），同时检查 APP_FEATURE_FILE 功能
async fn app_check_get(
    app_id: u64,
    is_edit: bool,
    auth_data: &UserAuthData,
    req_dao: &RequestDao,
    web_dao: &WebDao,
) -> JsonResult<AppModel> {
    let app = web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(app_id)
        .await?;

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
