use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacRoleStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacRoleResRange {
    Include = 1, //由RbacRoleModel决定,包含某些授权
    Any = 2,     //任意资源
    Exclude = 3, //由RbacRoleModel决定,排除某些授权
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacRoleUserRange {
    // Any = 1,    //任意用户
    // Logged = 2,    //登录用户
    Custom = 1,  //自定义用户 由RbacRoleUserModel决定
    Session = 2, //在会话时使用临时角色,任意用户,已登录用户为两个特殊的SESSION角色
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacResStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacOpStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacOpResStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacPermStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacRoleUserStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacAuditResult {
    Succ = 1,
    Fail = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum RbacAuditIs {
    Yes = 1,
    No = 0,
}
