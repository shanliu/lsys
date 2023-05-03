use lsys_core::now_time;
use lsys_logger::dao::ChangeLogData;

use rand::seq::SliceRandom;

fn range_client_key() -> String {
    const BASE_STR: &str = "0123456789abcdef0123456789abcdef";
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, 64)
            .cloned()
            .collect(),
    )
    .unwrap_or_else(|_| {
        format!(
            "{:x}",
            md5::compute(now_time().unwrap_or_default().to_string().as_bytes())
        )
    })
}

mod apps;
mod oauth;

pub use apps::*;
pub use oauth::*;
use serde::Serialize;

//日志
#[derive(Serialize)]
pub(crate) struct AppLog {
    pub action: &'static str,
    pub name: String,
    pub client_id: String,
    pub client_secret: String,
    pub callback_domain: String,
}

impl ChangeLogData for AppLog {
    fn log_type<'t>() -> &'t str {
        "setting"
    }
    fn format(&self) -> String {
        format!("{}:{}[{}]", self.action, self.name, self.client_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
