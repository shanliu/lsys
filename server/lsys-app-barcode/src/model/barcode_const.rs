use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum BarcodeCreateStatus {
    EnablePrivate = 1,
    EnablePublic = 2,
    Delete = 3,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum BarcodeParseStatus {
    Succ = 1,
    Fail = 2,
    Delete = 3,
}
