use std::sync::Arc;

use config::ConfigError;
use lsys_app::model::AppsModel;
use lsys_core::{AppCore, FluentMessage};
use lsys_sender::{
    dao::{
        AliyunSender, AliyunSenderTask, AliyunSmsRecord, SmsSender, SmsTaskAcquisition,
        SmsTaskRecord,
    },
    model::SenderSmsMessageModel,
};
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
    pub aliyun_sender: AliyunSender,
    smser: Arc<SmsSender<AliyunSmsRecord, ()>>,
    sms_record: SmsTaskRecord,
    fluent: Arc<FluentMessage>,
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
    ) -> Self {
        let acquisition = AliyunSmsRecord::new(app_core.clone(), db.clone());
        let sms_record = acquisition.sms_record().to_owned();
        let smser = Arc::new(SmsSender::new(
            app_core,
            redis,
            task_size,
            task_timeout,
            is_check,
            acquisition, //结构不大,不用Arc,引用,方便处理,不然生命周期太难搞
        ));
        let aliyun_sender = AliyunSender::new(db);
        Self {
            smser,
            fluent,
            sms_record,
            aliyun_sender,
        }
    }
    // 短信后台任务
    pub async fn task(&self) {
        self.smser
            .task::<_, _>(AliyunSenderTask::new(self.aliyun_sender.clone()))
            .await;
    }
    // 短信后台任务
    pub fn sms_record(&self) -> &SmsTaskRecord {
        &self.sms_record
    }
    // 短信发送接口
    #[allow(clippy::too_many_arguments)]
    pub async fn app_send(
        &self,
        app: &AppsModel,
        tpl_type: &str,
        mobile: &[String],
        body: &str,
        send_time: Option<u64>,
        cancel_key: &Option<String>,
    ) -> Result<(), WebAppSmserError> {
        let mb = mobile
            .iter()
            .map(|e| ("86", e.as_str()))
            .collect::<Vec<_>>();
        for tmp in mb.iter() {
            check_mobile(&self.fluent, tmp.0, tmp.1)
                .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        }
        self.smser
            .send(
                Some(app.id),
                &mb,
                tpl_type,
                body,
                &send_time,
                &Some(app.user_id),
                cancel_key,
            )
            .await
            .map_err(WebAppSmserError::System)
            .map(|_| ())
    }
    // 短信发送接口
    pub async fn app_send_cancel(
        &self,
        app: &AppsModel,
        cancel_key: &str,
    ) -> Result<(), WebAppSmserError> {
        self.smser
            .cancal_from_key(cancel_key, &app.user_id)
            .await
            .map_err(WebAppSmserError::System)
            .map(|_| ())
    }
    // 短信发送接口
    pub async fn send_cancel(
        &self,
        message: &SenderSmsMessageModel,
        user_id: u64,
    ) -> Result<(), WebAppSmserError> {
        self.smser
            .cancal_from_message(message, &user_id)
            .await
            .map_err(WebAppSmserError::System)
            .map(|_| ())
    }
    // 短信发送接口
    async fn send(
        &self,
        tpl_type: &str,
        area: &str,
        mobile: &str,
        body: &str,
    ) -> Result<(), WebAppSmserError> {
        check_mobile(&self.fluent, area, mobile)
            .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        self.smser
            .send(None, &[(area, mobile)], tpl_type, body, &None, &None, &None)
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
        self.send("valid_code", area, mobile, &context.into_json().to_string())
            .await
    }
}
