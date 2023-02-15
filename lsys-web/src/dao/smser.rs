use std::sync::Arc;

use config::ConfigError;
use lsys_core::{AppCore, FluentMessage};
use lsys_sender::dao::{AliyunSenderTask, AliyunSmsRecord, SmsSender};
use lsys_user::dao::account::{check_mobile, UserAccountError};
use sqlx::{MySql, Pool};
use tera::Context;

pub enum WebAppSmserError {
    Config(ConfigError),
    System(String),
    Tera(tera::Error),
}
impl ToString for WebAppSmserError {
    fn to_string(&self) -> String {
        match self {
            WebAppSmserError::Config(err) => {
                format!("config error:{}", err)
            }
            WebAppSmserError::System(err) => {
                format!("error:{}", err)
            }
            WebAppSmserError::Tera(err) => {
                format!("tpl error:{}", err)
            }
        }
    }
}
impl From<ConfigError> for WebAppSmserError {
    fn from(err: ConfigError) -> Self {
        WebAppSmserError::Config(err)
    }
}

impl From<tera::Error> for WebAppSmserError {
    fn from(err: tera::Error) -> Self {
        WebAppSmserError::Tera(err)
    }
}

impl From<WebAppSmserError> for UserAccountError {
    fn from(err: WebAppSmserError) -> Self {
        UserAccountError::System(err.to_string())
    }
}
pub struct WebAppSmser {
    smser: Arc<SmsSender<AliyunSmsRecord, ()>>,
    fluent: Arc<FluentMessage>,
    db: Pool<MySql>,
}

impl WebAppSmser {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
        try_num: usize,
    ) -> Self {
        let smser = Arc::new(SmsSender::new(
            app_core.clone(),
            redis,
            task_size,
            task_timeout,
            is_check,
            AliyunSmsRecord::new(try_num, app_core, db.clone()),
        ));
        Self { smser, fluent, db }
    }
    // 短信后台任务
    pub async fn task(&self) {
        self.smser
            .task::<_, _>(AliyunSenderTask::new(self.db.clone()))
            .await;
    }
    // 短信发送接口
    pub async fn send(
        &self,
        tpl_type: &str,
        area: &str,
        mobile: &str,
        body: &str,
    ) -> Result<(), WebAppSmserError> {
        check_mobile(&self.fluent, area, mobile)
            .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        self.smser
            .send(None, &[(area, mobile)], tpl_type, body, None, None, None)
            .await
            .map_err(WebAppSmserError::System)
            .map(|_| ())
    }
    pub async fn send_valid_code(
        &self,
        area: &str,
        mobile: &str,
        code: &str,
        ttl: &usize,
    ) -> Result<(), WebAppSmserError> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("time", &ttl);
        self.tpl_send(area, mobile, "valid_code", context).await
    }
    pub async fn tpl_send(
        &self,
        area: &str,
        mobile: &str,
        tpl_type: &str,
        context: Context,
    ) -> Result<(), WebAppSmserError> {
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
}
