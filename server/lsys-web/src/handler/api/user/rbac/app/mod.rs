mod audit;
mod check;
mod op;
mod res;
mod res_type;
mod role;
mod role_perm;
mod role_user;

use crate::{
    common::{JsonError, JsonResult, RequestDao, UserAuthQueryDao},
    dao::access::api::user::{CheckUserAppEdit, CheckUserAppView},
};
pub use audit::*;
pub use check::*;
use lsys_access::dao::AccessSession;
use lsys_app::model::AppModel;
use lsys_core::fluent_message;
use lsys_user::dao::UserAuthData;
pub use op::*;
pub use res::*;
pub use res_type::*;
pub use role::*;
pub use role_perm::*;
pub use role_user::*;

async fn parent_app_check(req_dao: &UserAuthQueryDao) -> JsonResult<UserAuthData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    if auth_data.session().user_app_id != 0 {
        return Err(JsonError::Message(fluent_message!("bad-audit-access")));
    }
    Ok(auth_data)
}

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
        .find_by_id(&app_id)
        .await?;

    if is_edit {
        req_dao
            .web_dao
            .web_rbac
            .check(
                &req_dao.req_env,
                Some(auth_data),
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
                &req_dao.req_env,
                Some(auth_data),
                &CheckUserAppView {
                    res_user_id: app.user_id,
                },
            )
            .await?;
    }
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_check(&app)
        .await?;
    Ok(app)
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

//@todo 子应用调试工具根据页面在定,待完善.......
