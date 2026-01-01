use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppStatus {
    Init = 1,    //正常
    Enable = 2,  //正常
    Disable = 3, //被禁用
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppFeatureStatus {
    Enable = 1,  //正常
    Disable = 2, //被禁用
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppOAuthServerScopeStatus {
    Enable = 1,  //正常
    Delete = -1, //删除
}

//用户请求类型
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppRequestType {
    AppReq = 1,           //新应用申请
    AppChange = 2,        //应该更改申请
    SubApp = 3,           //子应用可用申请
    ExterLogin = 4,       //外部账号登录系统申请
    OAuthServer = 5,      //Oauth服务申请
    OAuthClient = 6,      //Oauth登录申请
    OAuthClientScope = 7, //OAUTH登录新增权限申请
    ExterFeatuer = 8,     //外部功能申请:如发短信邮件等
}

impl AppRequestType {
    pub fn get_inner_feature() -> Vec<AppRequestType> {
        vec![
            AppRequestType::SubApp,
            AppRequestType::ExterLogin,
            AppRequestType::OAuthServer,
            AppRequestType::OAuthClient,
        ]
    }
    pub fn feature_key(&self) -> &str {
        match self {
            AppRequestType::AppReq => "",
            AppRequestType::AppChange => "",
            AppRequestType::OAuthClientScope => "",
            AppRequestType::ExterFeatuer => "feature",
            AppRequestType::OAuthClient => "oauth-client",
            AppRequestType::OAuthServer => "oauth-server",
            AppRequestType::SubApp => "sub-app",
            AppRequestType::ExterLogin => "ext-login",
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppRequestStatus {
    Pending = 1,  //待审
    Approved = 2, //批准
    Rejected = 3, //驳回
    Invalid = 4,  //作废
    Delete = -1,  //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppSecretType {
    App = 1,    //应用
    OAuth = 2,  //oauth
    Notify = 3, //回调
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppSecretStatus {
    Enable = 1,  //正常
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppNotifyDataStatus {
    Init = 1,
    Succ = 2,
    Fail = 3,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AppNotifyTryTimeMode {
    Fixed = 1,
    Exponential = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "u8")]
pub enum AppNotifyType {
    Http = 1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "u8")]
pub enum AppOAuthClientRefreshTokenStatus {
    Init = 1,
    Delete = -1,
}
