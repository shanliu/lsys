use std::sync::Arc;

use ip2location::Record;
use tokio::sync::Mutex;
use tracing::debug;

use crate::dao::login::AccountLoginEnv;

pub(crate) async fn env_to_city(
    lock_db: &Arc<Mutex<ip2location::DB>>,
    login_env: &AccountLoginEnv,
) -> Option<String> {
    let login_ip = login_env.login_ip?;
    #[allow(unused_mut)]
    let mut db = lock_db.lock().await;
    if let Some(ref ip) = login_env.login_ip {
        let bip = *ip;
        if let Ok(rec) = db.ip_lookup(bip) {
            match rec {
                Record::LocationDb(record) => {
                    debug!("parse city: {:?} on ip: {:?}", record, login_ip);
                    let city = [
                        record
                            .country
                            .map(|e| e.short_name.to_string())
                            .unwrap_or_default(),
                        record.region.unwrap_or_default().to_string(),
                        record.city.unwrap_or_default().to_string(),
                    ]
                    .into_iter()
                    .filter(|e| !e.is_empty() && *e != "-")
                    .collect::<Vec<String>>()
                    .join("-");
                    return Some(city);
                }
                Record::ProxyDb(_) => {}
            }
        }
    }
    None
}
