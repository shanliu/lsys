mod cache;
mod secret;
use crate::model::AppSecretModel;
pub use cache::*;
pub use secret::*;
use serde::{Deserialize, Serialize};
use std::convert::From;

#[derive(Clone, Serialize, Deserialize)]
pub struct AppSecretRecrod {
    pub secret_data: String,
    pub time_out: u64,
}

impl From<AppSecretModel> for AppSecretRecrod {
    fn from(s: AppSecretModel) -> Self {
        Self {
            secret_data: s.secret_data,
            time_out: s.time_out,
        }
    }
}
