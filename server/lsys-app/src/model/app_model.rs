use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use lsys_core::db::lsys_model;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app")]
pub struct AppModel {
    /// 用户ID
    #[sqlx(default)]
    pub id: u64,

    //上一级APP ID
    #[sqlx(default)]
    pub parent_app_id: u64,

    /// 名称
    #[sqlx(default)]
    pub name: String,

    /// ID
    #[sqlx(default)]
    pub client_id: String,

    /// 秘钥
    #[sqlx(default)]
    pub client_secret: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 申请用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 添加用户的APP id
    #[sqlx(default)]
    pub user_app_id: u64,

    /// 最后更新用户,审核,禁用时用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后更新用户,审核,禁用时时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app_feature")]
pub struct AppFeatureModel {
    #[sqlx(default)]
    pub id: u64,

    /// app id
    #[sqlx(default)]
    pub app_id: u64,

    /// client_secret
    #[sqlx(default)]
    pub feature_key: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

   /// 最后更新用户,审核,禁用时用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后更新用户,审核,禁用时时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app_oauth")]
pub struct AppOAuthClientModel {
    #[sqlx(default)]
    pub id: u64,

    /// app id
    #[sqlx(default)]
    pub app_id: u64,

    /// client_secret
    #[sqlx(default)]
    pub oauth_secret: String,
    
    /// callback_domain
    #[sqlx(default)]
    pub callback_domain: String,

    /// scope_data
    #[sqlx(default)]
    pub scope_data: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

   /// 最后更新用户,审核,禁用时用户
     #[sqlx(default)]
     pub change_user_id: u64,
 
      /// 最后更新用户,审核,禁用时时间
     #[sqlx(default)]
     pub change_time: u64,
}


#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app_oauth")]
pub struct AppOAuthServerScopeModel {
    #[sqlx(default)]
    pub id: u64,

    /// app id
    #[sqlx(default)]
    pub app_id: u64,

    /// scope_key
    #[sqlx(default)]
    pub scope_key: String,
    
    /// scope_name
    #[sqlx(default)]
    pub scope_name: String,

    /// scope_desc
    #[sqlx(default)]
    pub scope_desc: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 最后更新用户,审核,禁用时用户
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后更新用户,审核,禁用时时间
    #[sqlx(default)]
    pub change_time: u64,
}




#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app_request")]
pub struct AppRequestModel {
    #[sqlx(default)]
    pub id: u64,

    /// 冗余APP表parent_app_id 方便用于过滤数据
    #[sqlx(default)]
    pub parent_app_id: u64,
    
    /// app id
    #[sqlx(default)]
    pub app_id: u64,

    /// 请求类型
    #[sqlx(default)]
    pub request_type: i8,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 请求用户
    #[sqlx(default)]
    pub request_user_id: u64,

    /// 请求时间
    #[sqlx(default)]
    pub request_time: u64,

    /// 审核用户
    #[sqlx(default)]
    pub confirm_user_id: u64,

    /// 审核时间
    #[sqlx(default)]
    pub confirm_time: u64,

    /// 审核消息    
    #[sqlx(default)]
    pub confirm_note: String,
}

///应用申请OAUTH登录相关数据
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app_request_feature")]
pub struct AppRequestFeatureModel {
    #[sqlx(default)]
    pub id: u64,

    //请求ID
    #[sqlx(default)]
    pub app_request_id: u64,

    /// feature_data 数据
    #[sqlx(default)]
    pub feature_data: String,

}


///应用申请OAUTH登录相关数据
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app_request_change")]
pub struct AppRequestOAuthClientModel {
    #[sqlx(default)]
    pub id: u64,

    //请求ID
    #[sqlx(default)]
    pub app_request_id: u64,

    /// scope_data 数据
    #[sqlx(default)]
    pub scope_data: String,

}


/// 请求更改APP信息
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(db_type = "MySql", table_name = "app_request_set_info")]
pub struct AppRequestSetInfoModel {
    #[sqlx(default)]
    pub id: u64,

    //请求ID
    #[sqlx(default)]
    pub app_request_id: u64,

    /// 名称
    #[sqlx(default)]
    pub name: String,

    /// client_id
    #[sqlx(default)]
    pub client_id: String,
}
