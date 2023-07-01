use std::{collections::HashMap, sync::Arc};

use config::ConfigError;
use lsys_app::model::AppsModel;
use lsys_core::{AppCore, FluentMessage, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_sender::{
    dao::{
        AliYunSenderTask, HwYunSenderTask, SenderAliYunConfig, SenderError, SenderHwYunConfig,
        SenderTenYunConfig, SenderTplConfig, SmsRecord, SmsSender, TenyunSenderTask,
    },
    model::SenderSmsMessageModel,
};
use lsys_setting::dao::Setting;
use lsys_user::dao::account::{check_mobile, UserAccountError};
use serde_json::json;
use sqlx::{MySql, Pool};

pub enum WebAppSmserError {
    Config(ConfigError),
    System(String),
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
        }
    }
}
impl From<ConfigError> for WebAppSmserError {
    fn from(err: ConfigError) -> Self {
        WebAppSmserError::Config(err)
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
    aliyun_sender: SenderAliYunConfig,
    hwyun_sender: SenderHwYunConfig,
    tenyun_sender: SenderTenYunConfig,
    smser: Arc<SmsSender>,
    fluent: Arc<FluentMessage>,
}

impl WebAppSmser {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        setting: Arc<Setting>,
        logger: Arc<ChangeLogger>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let smser = Arc::new(SmsSender::new(
            app_core,
            redis,
            db,
            setting.clone(),
            fluent.clone(),
            logger,
            task_size,
            task_timeout,
            is_check,
        ));
        let aliyun_sender =
            SenderAliYunConfig::new(setting.multiple.clone(), smser.tpl_config.clone());
        let hwyun_sender =
            SenderHwYunConfig::new(setting.multiple.clone(), smser.tpl_config.clone());
        let tenyun_sender =
            SenderTenYunConfig::new(setting.multiple.clone(), smser.tpl_config.clone());

        Self {
            smser,
            fluent,
            aliyun_sender,
            hwyun_sender,
            tenyun_sender,
        }
    }
    pub fn tpl_config(&self) -> &SenderTplConfig {
        &self.smser.tpl_config
    }
    pub fn hwyun_sender(&self) -> &SenderHwYunConfig {
        &self.hwyun_sender
    }
    pub fn aliyun_sender(&self) -> &SenderAliYunConfig {
        &self.aliyun_sender
    }
    pub fn tenyun_sender(&self) -> &SenderTenYunConfig {
        &self.tenyun_sender
    }
    pub fn sms_record(&self) -> &SmsRecord {
        &self.smser.sms_record
    }
    // 短信后台任务
    pub async fn task(&self) -> Result<(), WebAppSmserError> {
        Ok(self
            .smser
            .task(vec![
                Box::<AliYunSenderTask>::default(),
                Box::<HwYunSenderTask>::default(),
                Box::<TenyunSenderTask>::default(),
            ])
            .await?)
    }
    // 短信发送接口
    #[allow(clippy::too_many_arguments)]
    pub async fn app_send(
        &self,
        app: &AppsModel,
        tpl_type: &str,
        mobile: &[String],
        body: &HashMap<String, String>,
        send_time: &Option<u64>,
        cancel_key: &Option<String>,
        max_try_num: &Option<u16>,
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
                &json!(body).to_string(),
                send_time,
                &Some(app.user_id),
                cancel_key,
                max_try_num,
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
        max_try_num: &Option<u16>,
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
                max_try_num,
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
        let mut context = HashMap::new();
        context.insert("code", code.to_owned());
        context.insert("time", ttl.to_string());
        self.send(
            "valid_code",
            area,
            mobile,
            &json!(context).to_string(),
            &None,
            env_data,
        )
        .await
    }
}
