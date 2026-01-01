//RBAC中 操作日志
use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

use super::user::RoleAddUser;

#[derive(Serialize)]
pub(crate) struct LogRole<'t> {
    pub action: &'t str,
    pub role_name: &'t str,
    pub role_key: &'t str,
    pub app_id: u64,
    pub user_range: i8,
    pub res_range: i8,
    pub user_id: u64,
}

impl ChangeLogData for LogRole<'_> {
    fn log_type() -> &'static str {
        "rbac-role"
    }
    fn message(&self) -> String {
        format!("{} {} ", self.action, self.role_name)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogRoleUser<'t> {
    pub action: &'t str,
    pub name: &'t str,
    pub add_user: Option<Vec<RoleAddUser>>,
    pub del_user: Option<Vec<u64>>,
    pub user_id: u64,
}

impl ChangeLogData for LogRoleUser<'_> {
    fn log_type() -> &'static str {
        "rbac-role-user"
    }
    fn message(&self) -> String {
        format!("{} :{} ", self.name, self.action,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogRolePerm<'t> {
    pub action: &'t str,
    pub name: &'t str,
    pub add_user: Option<Vec<(u64, u64)>>,
    pub del_user: Option<Vec<u64>>,
    pub user_id: u64,
}

impl ChangeLogData for LogRolePerm<'_> {
    fn log_type() -> &'static str {
        "rbac-role-perm"
    }
    fn message(&self) -> String {
        format!("{} :{} ", self.name, self.action,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
