use serde::{Deserialize, Serialize};
use sqlx_model::SqlxModelStatus;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacResStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacResOpStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacRoleOpPositivity {
    Allow = 1, //加权
    Deny = 0,  //减权
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacRoleUserRange {
    AllUser = 1,  //游客
    Login = 2,    //登录用户
    User = 3,     //指定用户 由RbacRoleUserModel决定
    Relation = 4, //指定关系角色
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacRoleResOpRange {
    AllowAll = 3, //允许所有权限[需单独授权]
    // AllowSelf = 2,   //允许自身资源
    AllowCustom = 1, //由RbacRoleOpModel决定[用户ID不等于当前访问用户,RbacRoleUser添加需单独授权]
    DenyAll = 0,     //禁止所有权限[需单独授权]
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacRoleStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacRoleUserStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacRoleOpStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacTagsSource {
    Role = 1,
    Res = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum RbacTagsStatus {
    Enable = 1,
    Delete = -1,
}
