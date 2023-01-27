use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::SqlxModel;

// 资源  属性: 1. 用户 2. 可执行操作【必须明确授权】
// 角色  访问用户对资源用户纬度: 1. 为指定关系 2.为特定分组关系 3.为任意用户

//需求
//用户 是否有对 资源【RbacResModel】 进行 资源某些操作的权限【RbacResOpModel】
//需明确:
//  资源 用户【全局，还是指定用户】

//核心流程
//用户 -> 角色 -> 资源(操作) -> 是否满足权限

//操作流程

//入参：
//  1. 进行访问用户id  @check的 user_id 参数
//  2. 需要访问资源及对该资源的操作,可能需要多个资源权限 @check的 check_vec 参数
//      1. 资源是系统还是某用户 【AccessRes.user_id】
//      2. 要资源的操作列表【【AccessRes.ops】 及这些操作的 默认是否需要授权【AccessResOp.must_authorize】
//      3. 涉及到多个权限时，存在任意一个未授权都认为未授权，即 check_vec 多个值
//  3. 资源所属用户 跟 进行访问用户 的角色关系key  @check的 relation_role 参数
//      1. 系统资源 关系key示例：会员等级
//      2. 特定用户资源 关系key示例：指定某些组或用户有查看权限
//  4. 访问某资源属于 访问用户 对该资源操作是否不检查权限 @check的 self_res_skip 参数

//  资源操作权限 默认是否需要授权：【AccessResOp.must_authorize】
//      1. 资源操作权限【RbacResModel】不存在资源记录即未创建
//          1. 默认需要授权【如后台页面：管理页面】，认为无权限
//          2. 默认不需要授权【如前台页面：登录，首页等】，认为有权限
//      2. 资源操作权限 存在记录 由角色管控
//  资源权限 分类：
//      1. 用户资源 user_id>0
//      2. 系统资源 user_id=0

//查询 进行访问用户 拥有角色
//  得到公共角色:
//      1. 系统资源 公共权限角色 即 【RbacRoleModel user_id=0】的角色
//      2. 如果为 被访问用户 的资源【资源user_id>0】,由被访问用户决定是否能被访问 进行访问用户
//         通过【传入自定义key:RbacRoleRelationModel + 该key的用户id，可以设置为 被访问用户ID】确定 被访问用户的关系
//        【传入自定义key】+  查 被访问用户 的对该关系用户 公共权限角色
//
//  得到 进行访问用户 被配置角色 通过 RbacRoleUserModel 获取

//合并 进行访问用户 拥有角色，根据 RbacRoleOpModel 得到 拥有的资源操作权限 和 禁止的资源操作权限
//在比较需要访问资源操作是否通过权限验证【存在任意一个禁止则无权操作】

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
//资源操作权限，如对某资源进行：查看 删除 编辑等
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
