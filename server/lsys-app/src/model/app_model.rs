use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::sqlx_model;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "app")]
pub struct AppsModel {
    /// 用户ID
    #[sqlx(default)]
    pub id: u64,

    /// 名称
    #[sqlx(default)]
    pub name: String,

    /// ID
    #[sqlx(default)]
    pub client_id: String,

    /// 秘钥
    #[sqlx(default)]
    pub client_secret: String,

    /// oauth 秘钥
    #[sqlx(default)]
    pub oauth_secret: String,

    #[sqlx(default)]
    pub callback_domain: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 申请用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 密码ID  default:  0
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 添加时间
    #[sqlx(default)]
    pub change_time: u64,

    /// 确认用户ID
    #[sqlx(default)]
    pub confirm_user_id: u64,

    /// 确认时间
    #[sqlx(default)]
    pub confirm_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "app_sub_user")]
pub struct AppSubUserModel {
    #[sqlx(default)]
    pub id: u64,

    /// app id
    #[sqlx(default)]
    pub app_id: u64,

    /// sub app id
    #[sqlx(default)]
    pub user_id: u64,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 确认时间时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "app_sub_app")]
pub struct AppSubAppsModel {
    #[sqlx(default)]
    pub id: u64,

    /// sub app id
    #[sqlx(default)]
    pub user_id: u64,

    /// app id
    #[sqlx(default)]
    pub app_id: u64,

    /// app id
    #[sqlx(default)]
    pub sub_app_id: u64,

    /// client_secret
    #[sqlx(default)]
    pub sub_client_secret: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 确认时间时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "app_oauth_token")]
pub struct AppsTokenModel {
    #[sqlx(default)]
    pub id: u64,

    /// app id
    #[sqlx(default)]
    pub app_id: u64,

    /// 访问用户
    #[sqlx(default)]
    pub access_user_id: u64,

    /// token
    #[sqlx(default)]
    pub code: String,

    /// token
    #[sqlx(default)]
    pub token: String,

    /// scope
    #[sqlx(default)]
    pub scope: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 授权时间
    #[sqlx(default)]
    pub token_time: u64,

    /// 过期时间
    #[sqlx(default)]
    pub timeout: u64,
}
