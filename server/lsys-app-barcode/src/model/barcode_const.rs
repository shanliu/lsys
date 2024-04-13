use serde::{Deserialize, Serialize};
use sqlx_model::sqlx_model_status;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "i8")]
pub enum BarcodeCreateStatus {
    EnablePrivate = 1,
    EnablePublic = 2,
    Delete = 3,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "i8")]
pub enum BarcodeParseStatus {
    Succ = 1,
    Fail = 2,
    Delete = 3,
}
