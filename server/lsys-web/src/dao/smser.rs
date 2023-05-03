use std::sync::Arc;

use config::ConfigError;
use lsys_app::model::AppsModel;
use lsys_core::{AppCore, FluentMessage, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_sender::{
    dao::{
        AliyunSender, AliyunSenderTask, SenderError, SmsSender, SmsTaskAcquisitionRecord,
        SmsTaskRecord,
    },
    model::SenderSmsMessageModel,
};
use lsys_setting::dao::MultipleSetting;
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
impl From<SenderError> for WebAppSmserError {
    fn from(err: SenderError) -> Self {
        WebAppSmserError::System(err.to_string())
    }
}

impl From<WebAppSmserError> for UserAccountError {
    fn from(err: WebAppSmserError) -> Self {
        UserAccountError::System(err.to_string())
    }
}

pub struct WebAppSmser {
    pub aliyun_sender: AliyunSender,
    smser: Arc<SmsSender<SmsTaskAcquisitionRecord, ()>>,
    app_core: Arc<AppCore>,
    sms_record: Arc<SmsTaskRecord>,
    fluent: Arc<FluentMessage>,
    setting: Arc<MultipleSetting>,
    db: Pool<MySql>,
}

impl WebAppSmser {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        setting: Arc<MultipleSetting>,
        logger: Arc<ChangeLogger>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let sms_record = Arc::new(SmsTaskRecord::new(
            db.clone(),
            app_core.clone(),
            fluent.clone(),
            logger,
        ));
        let acquisition = SmsTaskAcquisitionRecord::new(sms_record.clone());
        let smser = Arc::new(SmsSender::new(
            redis,
            task_size,
            task_timeout,
            is_check,
            acquisition,
        ));
        let aliyun_sender = AliyunSender::new(db.clone(), setting.clone());
        Self {
            app_core,
            smser,
            fluent,
            sms_record,
            aliyun_sender,
            setting,
            db,
        }
    }
    // 短信后台任务
    pub async fn task(&self) -> Result<(), WebAppSmserError> {
        Ok(self
            .smser
            .task(
                self.app_core.clone(),
                self.sms_record.clone(),
                vec![Box::new(AliyunSenderTask::new(AliyunSender::new(
                    self.db.clone(),
                    self.setting.clone(),
                )))],
            )
            .await?)
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
        env_data: Option<&RequestEnv>,
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
                env_data,
            )
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))
            .map(|_| ())
    }
    // APP 短信短信取消发送
    pub async fn app_send_cancel(
        &self,
        app: &AppsModel,
        cancel_key: &str,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppSmserError> {
        self.smser
            .cancal_from_key(cancel_key, &app.user_id, env_data)
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))
            .map(|_| ())
    }
    // 通过消息取消发送
    pub async fn send_cancel(
        &self,
        message: &SenderSmsMessageModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppSmserError> {
        self.smser
            .cancal_from_message(message, &user_id, env_data)
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))
            .map(|_| ())
    }
    // 短信发送接口
    async fn send(
        &self,
        tpl_type: &str,
        area: &str,
        mobile: &str,
        body: &str,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppSmserError> {
        check_mobile(&self.fluent, area, mobile)
            .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        self.smser
            .send(
                None,
                &[(area, mobile)],
                tpl_type,
                body,
                &None,
                &None,
                &None,
                env_data,
            )
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))
            .map(|_| ())
    }
    pub async fn send_valid_code(
        &self,
        area: &str,
        mobile: &str,
        code: &str,
        ttl: &usize,
        env_data: Option<&RequestEnv>,
    ) -> Result<(), WebAppSmserError> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("time", &ttl);
        self.send(
            "valid_code",
            area,
            mobile,
            &context.into_json().to_string(),
            env_data,
        )
        .await
    }
}
