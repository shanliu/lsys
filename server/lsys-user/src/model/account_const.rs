use serde::{Deserialize, Serialize};
use sqlx_model::SqlxModelStatus;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum UserStatus {
    Enable = 2,
    Init = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum UserEmailStatus {
    Init = 1,
    Valid = 2,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum UserMobileStatus {
    Init = 1,
    Valid = 2,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum UserNameStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum UserExternalStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum UserAddressStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum UserIndexStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq, Hash)]
#[sqlx_model_status(type = "i8")]
pub enum UserIndexCat {
    Address = 1,      //多个
    Email = 2,        //多个
    Mobile = 3,       //多个
    ExternalType = 4, //多个
    UserName = 5,     //且只存在其中一个
    NikeName = 6,     //只有 enable 跟 init, 且只存在其中一个
    UserStatus = 7,   //只有 enable 跟 init, 且只存在其中一个
    RegFrom = 8,      //如果存在其中一个
}
