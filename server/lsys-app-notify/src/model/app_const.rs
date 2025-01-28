use serde::{Deserialize, Serialize};
use lsys_core::db::lsys_model_status;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum NotifyDataStatus {
    Init = 1,
    Succ = 2,
    Fail = 3,
}
