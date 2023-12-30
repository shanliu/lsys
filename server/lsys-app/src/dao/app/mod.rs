use lsys_core::{rand_str, RandType};
use lsys_logger::dao::ChangeLogData;

// use rand::seq::SliceRandom;

fn range_client_key() -> String {
    rand_str(RandType::LowerHex, 64)
    // const BASE_STR: &str = "0123456789abcdef0123456789abcdef";
    // let mut rng = &mut rand::thread_rng();
    // String::from_utf8(
    //     BASE_STR
    //         .as_bytes()
    //         .choose_multiple(&mut rng, 64)
    //         .cloned()
    //         .collect(),
    // )
    // .unwrap_or_else(|_| {
    //     format!(
    //         "{:x}",
    //         md5::compute(now_time().unwrap_or_default().to_string().as_bytes())
    //     )
    // })
}

mod apps;
mod oauth;
mod sub_apps;

pub use apps::*;
pub use oauth::*;
use serde::Serialize;
pub use sub_apps::*;

//日志
#[derive(Serialize)]
pub(crate) struct AppLog {
    pub action: &'static str,
    pub name: String,
    pub status: i8,
    pub client_id: String,
    pub client_secret: String,
    pub callback_domain: String,
}

impl ChangeLogData for AppLog {
    fn log_type<'t>() -> &'t str {
        "app"
    }
    fn message(&self) -> String {
        format!("{}:{}[{}]", self.action, self.name, self.client_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct AppSubUserLog {
    pub used: bool,
    pub sub_app_user_id: u64,
}

impl ChangeLogData for AppSubUserLog {
    fn log_type<'t>() -> &'t str {
        "app-sub-user"
    }
    fn message(&self) -> String {
        format!(
            "{}:{}",
            if self.used { "enable" } else { "disable" },
            self.sub_app_user_id,
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Serialize)]
pub(crate) struct AppSubAppLog {
    pub parent_app_id: u64,
    pub sub_client_secret: Option<String>,
    pub status: i8,
}

impl ChangeLogData for AppSubAppLog {
    fn log_type<'t>() -> &'t str {
        "app-sub-user"
    }
    fn message(&self) -> String {
        format!("parent app {}", self.parent_app_id,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
