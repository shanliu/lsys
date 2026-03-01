use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_mfa_totp")]
pub struct MfaTotpModel {
    #[sqlx(default)]
    pub id: u64,

    #[sqlx(default)]
    pub app_id: u64,

    #[sqlx(default)]
    pub user_data: String,

    /// 1: enabled, 0: disabled
    #[sqlx(default)]
    pub status: i8,

    /// Base32 secret (or encrypted/encoded secret string if you later add encryption)
    #[sqlx(default)]
    pub secret_data: String,

    /// last accepted time-step (e.g. floor(now/30))
    #[sqlx(default)]
    pub last_used_step: u64,

    #[sqlx(default)]
    pub last_used_time: u64,

    #[sqlx(default)]
    pub add_time: u64,

    #[sqlx(default)]
    pub change_time: u64,
}
