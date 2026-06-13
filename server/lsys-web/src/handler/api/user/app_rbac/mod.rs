mod audit;
mod check;
mod mapping;
mod op;
mod res;
mod res_type;
mod role;
mod role_perm;
mod role_user;

use crate::{
    common::{JsonError, JsonResult, RequestDao, UserAuthQueryDao},
    dao::{
        WebDao,
        access::{
            RbacAccessCheckEnv,
            api::system::user::{CheckUserAppEdit, CheckUserAppView},
        },
    },
};
pub use audit::*;
pub use check::*;
use lsys_access::dao::AccessSession;
use lsys_app::model::AppModel;
use lsys_core::fluent_message;
use lsys_user::dao::UserAuthData;
pub use mapping::*;
pub use op::*;
pub use res::*;
pub use res_type::*;
pub use role::*;
pub use role_perm::*;
pub use role_user::*;

async fn parent_app_check(auth_dao: &UserAuthQueryDao) -> JsonResult<UserAuthData> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    if auth_data.session().user_app_id != 0 {
        return Err(JsonError::Message(fluent_message!("bad-audit-access")));
    }
    Ok(auth_data)
}

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
        .inner_feature_sub_app_check(&app)
        .await?;
    Ok(app)
}

async fn inner_user_data_to_user_id(
    app: &AppModel,
    use_app_user: bool,
    user_data: Option<&str>,
    web_dao: &WebDao,
) -> JsonResult<u64> {
    if use_app_user {
        return Ok(app.user_id);
    }
    Ok(web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, user_data.unwrap_or_default(), None, None)
        .await?
        .id)
}

//@todo 子应用调试工具根据页面在定,待完善.......
