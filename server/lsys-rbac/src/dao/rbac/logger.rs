use std::fmt::Display;

use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

use crate::model::{RbacRoleResOpRange, RbacRoleUserRange, RbacTagsSource};

use super::{ResOp, RoleAddUser, RoleSetOp};

#[derive(Serialize)]
pub(crate) struct LogTag {
    pub from_source: RbacTagsSource,
    pub action: &'static str,
    pub tags: Option<Vec<String>>,
}

impl ChangeLogData for LogTag {
    fn log_type<'t>() -> &'t str {
        "rbac-tag"
    }
    fn message(&self) -> String {
        match self.from_source {
            RbacTagsSource::Role => format!("{} role tag ", self.action),
            RbacTagsSource::Res => format!("{} res tag ", self.action),
        }
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogRes {
    pub action: &'static str,
    pub name: String,
    pub res_key: String,
}

impl ChangeLogData for LogRes {
    fn log_type<'t>() -> &'t str {
        "rbac-res"
    }
    fn message(&self) -> String {
        format!("{} {} [{}]", self.action, self.name, self.res_key)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogResOp {
    pub name: String,
    pub key: String,
    pub ops: Vec<ResOp>,
}

impl ChangeLogData for LogResOp {
    fn log_type<'t>() -> &'t str {
        "rbac-res-op"
    }
    fn message(&self) -> String {
        format!("set op {} {} [{:?}]", self.name, self.key, self.ops)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogRole {
    pub action: &'static str,
    pub name: String,
    pub relation_key: String,
    pub priority: i8,
    pub user_range: RbacRoleUserRange,
    pub res_op_range: RbacRoleResOpRange,
}

impl ChangeLogData for LogRole {
    fn log_type<'t>() -> &'t str {
        "rbac-role"
    }
    fn message(&self) -> String {
        format!("{} {} ", self.action, self.name)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize, Debug)]
pub(crate) enum LogRoleUserAction {
    Add,
    Del,
}
impl Display for LogRoleUserAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                LogRoleUserAction::Add => "add",
                LogRoleUserAction::Del => "del",
            }
        )
    }
}

#[derive(Serialize)]
pub(crate) struct LogRoleUser {
    pub action: LogRoleUserAction,
    pub name: String,
    pub add_user: Option<Vec<RoleAddUser>>,
    pub del_user: Option<Vec<u64>>,
}

impl ChangeLogData for LogRoleUser {
    fn log_type<'t>() -> &'t str {
        "rbac-role-user"
    }
    fn message(&self) -> String {
        format!(
            "{} {} :{} ",
            self.action,
            self.name,
            match self.action {
                LogRoleUserAction::Add => {
                    format!("add user:{:?}", self.add_user)
                }
                LogRoleUserAction::Del => {
                    format!("del user:{:?}", self.del_user)
                }
            }
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct LogRoleOp {
    pub name: String,
    pub role_op_vec: Vec<RoleSetOp>,
}

impl ChangeLogData for LogRoleOp {
    fn log_type<'t>() -> &'t str {
        "rbac-role-op"
    }
    fn message(&self) -> String {
        format!("ser role op {:?} user:{:?} ", self.name, self.role_op_vec)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
