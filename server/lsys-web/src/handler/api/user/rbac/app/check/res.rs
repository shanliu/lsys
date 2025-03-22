use crate::common::{JsonError, JsonResult, PageParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;
use lsys_access::model::UserModel;
use lsys_core::fluent_message;
use lsys_rbac::{
    dao::{AccessPermRow, AccessSessionRole, AccessUserFromRes},
    model::RbacRoleResRange,
};
pub struct AppResUserFromUserParam {
    pub app_id: u64,
    pub access_user_id: u64,
    pub page: Option<PageParam>,
}
//1 得到用户列表
pub async fn app_res_user_from_user(
    param: &AppResUserFromUserParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<UserModel>, bool, i64)> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    if auth_data.session().user_app_id != 0 {
        return Err(JsonError::Message(fluent_message!("bad-audit-access")));
    }
    let audit_user = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .find_by_id(&param.access_user_id)
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&audit_user.app_id)
        .await?;
    if app.user_id != auth_data.user_id() {
        return Err(JsonError::Message(fluent_message!("bad-audit-access")));
    }

    let mut user_ids = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_user_list_from_user(
            param.access_user_id,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let is_system = user_ids.contains(&0);
    user_ids.retain(|x| *x != 0);
    let user_data = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_users_by_ids(&user_ids)
        .await?
        .into_array();
    let count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_user_count_from_user(param.access_user_id)
        .await?;
    Ok((user_data, is_system, count))
}

pub struct AppResInfoFromUserData {
    pub app_id: u64,
    pub access_user_id: u64,
}

//2 根据用户查找最近授权详细
pub async fn app_res_info_from_user(
    param: &AppResInfoFromUserData,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<AccessUserFromRes> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_data_from_custom_user(auth_data.user_id(), param.access_user_id)
        .await?;
    Ok(res_data)
}

pub struct AppResListFromUserParam {
    pub app_id: u64,
    pub access_user_id: u64,
    pub role_user_id: u64,
    pub res_range: i8,
    pub page: Option<PageParam>,
}

//3 如果配置关系,查询具体的配置授权
pub async fn app_res_list_from_user(
    param: &AppResListFromUserParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<AccessPermRow>, i64)> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    if auth_data.session().user_app_id != 0 {
        return Err(JsonError::Message(fluent_message!("bad-audit-access")));
    }
    let res_range = RbacRoleResRange::try_from(param.res_range)?;
    let prem_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_list_from_custom_user(
            param.access_user_id,
            param.role_user_id,
            res_range,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_count_from_custom_user(param.access_user_id, param.role_user_id, res_range)
        .await?;
    Ok((prem_data, count))
}

pub struct AppResListFromSessionParam {
    pub app_id: u64,
    pub role_key: String,
    pub user_id: u64,
    pub page: Option<PageParam>,
}
//3 如果是会话角色,根据会话角色查询该会话角色的授权资源
pub async fn app_res_info_from_session(
    param: &AppResListFromSessionParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(bool, Vec<AccessPermRow>, i64)> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    if auth_data.session().user_app_id != 0 {
        return Err(JsonError::Message(fluent_message!("bad-audit-access")));
    }
    let rs = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_range_from_session_role(&AccessSessionRole {
            role_key: &param.role_key,
            user_id: param.user_id,
            app_id: param.app_id,
        })
        .await?;
    let mut all_res = false;
    let mut prem_data = vec![];
    let mut count = 0;
    match rs {
        ref d @ (RbacRoleResRange::Include | RbacRoleResRange::Exclude) => {
            prem_data = req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .access
                .find_res_list_from_session_role(
                    &AccessSessionRole {
                        role_key: &param.role_key,
                        user_id: param.user_id,
                        app_id: param.app_id,
                    },
                    *d,
                    param.page.as_ref().map(|e| e.into()).as_ref(),
                )
                .await?;
            count = req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .access
                .find_res_count_from_session_role(
                    &AccessSessionRole {
                        role_key: &param.role_key,
                        user_id: param.user_id,
                        app_id: param.app_id,
                    },
                    *d,
                )
                .await?;
        }
        RbacRoleResRange::Any => {
            all_res = true;
        }
    }
    Ok((all_res, prem_data, count))
}
