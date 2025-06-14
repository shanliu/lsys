use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SessionStatus {
    Enable = 1,
    Delete = 2,
}
