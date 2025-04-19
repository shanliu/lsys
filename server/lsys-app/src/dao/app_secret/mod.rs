mod cache;
mod secret;
use crate::model::AppSecretModel;
use std::convert::From;

pub use cache::*;
pub use secret::*;

#[derive(Clone)]
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
