use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
//角色
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_role")]
pub struct RbacRoleModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub user_id: u64,

    ///应用ID,当user_id时,对应关联的应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 角色标识
    #[sqlx(default)]
    pub role_key: String,

    /// 用户范围
    #[sqlx(default)]
    pub user_range: i8,

    /// 资源可操作范围
    #[sqlx(default)]
    pub res_range: i8,

    /// 资源名称
    #[sqlx(default)]
    pub role_name: String,

    /// 状态 1 启用 -1 删除
    #[sqlx(default)]
    pub status: i8,

    /// 添加用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,
}

//资源
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_res")]
pub struct RbacResModel {
    #[sqlx(default)]
    pub id: u64,

    /// 资源类型
    #[sqlx(default)]
    pub res_type: String,

    /// 资源数据
    #[sqlx(default)]
    pub res_data: String,

    /// 用户ID 0 为系统资源
    #[sqlx(default)]
    pub user_id: u64,

    ///应用ID,当user_id时,对应关联的应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 资源名称
    #[sqlx(default)]
    pub res_name: String,

    /// 状态 1 启用 -1 删除
    #[sqlx(default)]
    pub status: i8,

    /// 最后修改用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,
}

//操作
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_op")]
pub struct RbacOpModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID 0 为系统资源
    #[sqlx(default)]
    pub user_id: u64,

    ///应用ID,当user_id时,对应关联的应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 资源操作KEY
    #[sqlx(default)]
    pub op_key: String,

    /// 资源操作名
    #[sqlx(default)]
    pub op_name: String,

    /// 状态 1 启用 -1 删除
    #[sqlx(default)]
    pub status: i8,

    /// 最后修改用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,
}

///操作跟资源关联
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_op_res")]
pub struct RbacOpResModel {
    #[sqlx(default)]
    pub id: u64,

    /// 资源操作ID
    #[sqlx(default)]
    pub op_id: u64,

    /// 资源类型
    #[sqlx(default)]
    pub res_type: String,

    /// 用户ID
    #[sqlx(default)]
    pub user_id: u64,

    ///应用ID,当user_id时,对应关联的应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 状态 1 启用 -1 删除
    #[sqlx(default)]
    pub status: i8,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,

    /// 绑定时间
    #[sqlx(default)]
    pub change_user_id: u64,
}

//资源可进行操作，如对某资源进行：查看 删除 编辑等
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_perm")]
pub struct RbacPermModel {
    #[sqlx(default)]
    pub id: u64,

    ///角色ID
    #[sqlx(default)]
    pub role_id: u64,

    ///资源id
    #[sqlx(default)]
    pub res_id: u64,

    ///资源id
    #[sqlx(default)]
    pub op_id: u64,

    /// 状态 1 启用 -1 删除
    /// 启用都认为管控
    /// 删除或不存在认为不管控
    #[sqlx(default)]
    pub status: i8,

    /// 添加用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,
}

/// 角色关联用户  
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_role_user")]
pub struct RbacRoleUserModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub role_id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub user_id: u64,

    /// 超时时间
    #[sqlx(default)]
    pub timeout: u64,

    /// 状态 1 启用 -1 删除
    #[sqlx(default)]
    pub status: i8,

    /// 添加用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,
}

///授权审计记录
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_audit")]
pub struct RbacAuditModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub user_id: u64,

    /// 授权检查结果
    #[sqlx(default)]
    pub role_key_data: String,

    /// 授权结果 0 失败 1 成功
    #[sqlx(default)]
    pub check_result: i8,

    /// 授权token
    #[sqlx(default)]
    pub token_data: String,

    /// 授权检查结果
    #[sqlx(default)]
    pub user_ip: String,

    /// 授权检查结果
    #[sqlx(default)]
    pub user_app_id: u64,

    /// 授权检查结果
    #[sqlx(default)]
    pub device_id: String,

    /// 授权检查结果
    #[sqlx(default)]
    pub device_name: String,

    /// 请求ID
    #[sqlx(default)]
    pub request_id: String,

    /// 绑定时间
    #[sqlx(default)]
    pub add_time: u64,
}

///授权审计详细记录
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "rbac_audit_detail")]
pub struct RbacAuditDetailModel {
    #[sqlx(default)]
    pub id: u64,

    /// 审计ID
    #[sqlx(default)]
    pub rbac_audit_id: u64,

    /// 资源类型
    #[sqlx(default)]
    pub res_type: String,

    /// 资源数据
    #[sqlx(default)]
    pub res_data: String,

    /// 资源ID
    #[sqlx(default)]
    pub res_user_id: u64,

    /// 资源操作KEY
    #[sqlx(default)]
    pub op_key: String,

    /// 资源ID
    #[sqlx(default)]
    pub res_id: u64,

    /// 操作ID
    #[sqlx(default)]
    pub op_id: u64,

    /// 授权结果 0 失败 1 成功
    #[sqlx(default)]
    pub check_result: i8,

    /// 资源是否要授权访问
    #[sqlx(default)]
    pub res_auth: i8,

    /// 是否超级用户角色
    #[sqlx(default)]
    pub is_role_excluce: i8,

    /// 是否超级用户角色
    #[sqlx(default)]
    pub is_role_include: i8,

    /// 是否超级用户角色
    #[sqlx(default)]
    pub is_role_all: i8,

    /// 是否超级用户角色
    #[sqlx(default)]
    pub role_data: String,

    /// 绑定时间
    #[sqlx(default)]
    pub add_time: u64,
}
