use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::SqlxModel;

//资源
#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "rbac_res")]
pub struct RbacResModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID 0 为系统资源
    #[sqlx(default)]
    pub user_id: u64,

    /// 资源名称
    #[sqlx(default)]
    pub name: String,

    /// 资源标识
    #[sqlx(default)]
    pub res_key: String,

    /// 状态 1 启用 -1 删除
    #[sqlx(default)]
    pub status: i8,

    /// 添加用户
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 添加用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 绑定时间
    #[sqlx(default)]
    pub change_time: u64,
}
//资源可进行操作，如对某资源进行：查看 删除 编辑等
#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "rbac_res_op")]
pub struct RbacResOpModel {
    #[sqlx(default)]
    pub id: u64,

    /// 资源操作名称
    #[sqlx(default)]
    pub name: String,

    /// 资源操作key
    #[sqlx(default)]
    pub op_key: String,

    ///资源id
    #[sqlx(default)]
    pub res_id: u64,

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
//角色
#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "rbac_role")]
pub struct RbacRoleModel {
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub user_id: u64,

    /// 资源名称
    #[sqlx(default)]
    pub name: String,

    /// 预定关系标识 [代码中标记]
    #[sqlx(default)]
    pub relation_key: String,

    /// 优先级
    #[sqlx(default)]
    pub priority: i8,

    /// 用户范围 1 任意用户 2 登录用户 3  指定用户【RbacRoleUserModel】
    #[sqlx(default)]
    pub user_range: i8,

    /// 资源操作范围 0 默认，由【RbacRoleOpModel】决定 1 开放所有权限 -1 禁止所有权限：屏蔽用户
    #[sqlx(default)]
    pub res_op_range: i8,

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

/// 角色关联用户  
#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "rbac_role_user")]
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
/// 角色的可进行操作 关联 角色【RbacRoleModel】跟资源【RbacResOpModel】操作  
#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "rbac_role_op")]
pub struct RbacRoleOpModel {
    #[sqlx(default)]
    pub id: u64,

    /// 资源操作id
    #[sqlx(default)]
    pub res_op_id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub role_id: u64,

    /// 加权【存在角色有权限，一般角色】 OR 减权【存在角色没权限，如:黑名单】
    #[sqlx(default)]
    pub positivity: i8,

    /// 启用时间规则
    // #[sqlx(default)]
    // pub time_rule: String,

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

/// 给角色 资源分组用的tag
#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "rbac_tags")]
pub struct RbacTagsModel {
    #[sqlx(default)]
    pub id: u64,

    /// 来源表
    #[sqlx(default)]
    pub from_source: i8,

    /// 来源表id
    #[sqlx(default)]
    pub from_id: u64,

    /// 用户id
    #[sqlx(default)]
    pub user_id: u64,

    /// TAG名称
    #[sqlx(default)]
    pub name: String,

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
