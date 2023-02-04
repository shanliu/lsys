use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use config::ConfigError;
use lsys_core::{AppCore, FluentMessage};
use tera::Context;
use tokio::{sync::oneshot::Receiver, time::sleep};
use tracing::log::warn;

pub enum SmserError {
    Config(ConfigError),
    System(String),
    Tera(tera::Error),
}
impl ToString for SmserError {
    fn to_string(&self) -> String {
        match self {
            SmserError::Config(err) => {
                format!("config error:{}", err)
            }
            SmserError::System(err) => {
                format!("error:{}", err)
            }
            SmserError::Tera(err) => {
                format!("tpl error:{}", err)
            }
        }
    }
}
impl From<ConfigError> for SmserError {
    fn from(err: ConfigError) -> Self {
        SmserError::Config(err)
    }
}

pub trait SmserSender {
    fn send(&self, tpl_type: &str, area: &str, mobile: &str, body: &str) -> Result<(), SmserError>;
}
pub struct Smser {
    app_core: Arc<AppCore>,
    sender: Box<dyn SmserSender>,
    fluent: Arc<FluentMessage>,
    task_last_id: Arc<AtomicU64>,
}

impl Smser {
    pub fn new(
        app_core: Arc<AppCore>,
        sender: Box<dyn SmserSender>,
        fluent: Arc<FluentMessage>,
    ) -> Self {
        Self {
            app_core,
            sender,
            fluent,
            task_last_id: Arc::new(AtomicU64::new(0)),
        }
    }
    pub async fn send(
        &self,
        _tpl_type: &str,
        _area: &str,
        _mobile: &str,
        _body: &str,
    ) -> Result<(), SmserError> {
        let _ = self.app_core;
        let _ = self.sender;
        let _ = self.fluent;
        Ok(())
    }
    pub async fn tpl_send(
        &self,
        area: &str,
        mobile: &str,
        tpl_type: &str,
        context: Context,
    ) -> Result<(), SmserError> {
        // # [sms_notify.valid_code]
        // # tpl="sms/valid_code.txt"
        // # sms_tpl="valid_code"
        // let notifys = self.app_core.config.get_table("sms_notify")?;
        // let notify = notifys
        //     .get(tpl_type)
        //     .ok_or_else(|| {
        //         err_result!(format!("not find {} notify config [sms_notify]", tpl_type))
        //     })?
        //     .to_owned()
        //     .into_table()
        //     .map_err(|e| err_result!(e.to_string() + "[sms_notify_parse]"))?;
        // let template_name = notify
        //     .get("tpl")
        //     .ok_or_else(|| err_result!(format!("not find tpl on {}", tpl_type)))?
        //     .to_owned()
        //     .into_string()
        //     .map_err(|e| err_result!(e.to_string() + "[sms_notify_tpl]"))?;
        // let body = self.tera.render(&template_name, context)?;
        self.send(tpl_type, area, mobile, &context.into_json().to_string())
            .await
    }
    pub async fn task(&self, rx: Receiver<u64>) {
        // let (tx, rx) = oneshot::channel::<u64>();
        let task_id = self.task_last_id.clone();
        tokio::spawn(async move {
            match rx.await {
                Ok(v) => {
                    if v > task_id.fetch_max(v, Ordering::Relaxed) {
                        task_id.store(v, Ordering::Relaxed)
                    }
                }
                Err(err) => {
                    warn!("sms send task err:{}", err);
                    sleep(Duration::from_secs(10)).await;
                }
            }
        });
    }
}
