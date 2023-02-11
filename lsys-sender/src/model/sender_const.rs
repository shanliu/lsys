use serde::{Deserialize, Serialize};
use sqlx_model::SqlxModelStatus;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsMessageStatus {
    Init = 0,
    IsSend = 1,
    SendFail = 2,
}
