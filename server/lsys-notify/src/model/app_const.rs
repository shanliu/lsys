use serde::{Deserialize, Serialize};
use sqlx_model::sqlx_model_status;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "i8")]
pub enum NotifyDataStatus {
    Init = 1,
    Succ = 2,
    Fail = 3,
}
