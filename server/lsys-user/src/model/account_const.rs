use serde::{Deserialize, Serialize};
use lsys_core::db::lsys_model_status;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountStatus {
    Enable = 2,
    Init = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountEmailStatus {
    Init = 1,
    Valid = 2,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountMobileStatus {
    Init = 1,
    Valid = 2,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountNameStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountExternalStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountAddressStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountIndexStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[lsys_model_status(field_type = "i8")]
pub enum AccountIndexCat {
    Address = 1,      //多个
    Email = 2,        //多个
    Mobile = 3,       //多个
    ExternalType = 4, //多个
    AccountName = 5,     //且只存在其中一个
    NikeName = 6,     //只有 enable 跟 init, 且只存在其中一个
    AccountStatus = 7,   //只有 enable 跟 init, 且只存在其中一个
    RegFrom = 8,      //如果存在其中一个
}
