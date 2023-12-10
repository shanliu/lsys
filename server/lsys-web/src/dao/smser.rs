use std::{collections::HashMap, sync::Arc};

use config::ConfigError;
use lsys_app::model::AppsModel;
use lsys_core::{AppCore, FluentMessage, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_notify::dao::Notify;
use lsys_sender::{
    dao::{
        AliYunSendStatus, AliYunSenderTask, CloOpenSenderTask, HwYunSenderTask, JDCloudSenderTask,
        JDSendStatus, NetEaseSendStatus, NetEaseSenderTask, SenderAliYunConfig,
        SenderCloOpenConfig, SenderError, SenderHwYunConfig, SenderJDCloudConfig,
        SenderNetEaseConfig, SenderTenYunConfig, SmsSender, TenYunSendStatus, TenyunSenderTask,
    },
    model::{SenderSmsBodyModel, SenderSmsMessageModel},
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
    pub aliyun_sender: SenderAliYunConfig,
    pub hwyun_sender: SenderHwYunConfig,
    pub tenyun_sender: SenderTenYunConfig,
    pub cloopen_sender: SenderCloOpenConfig,
    pub netease_sender: SenderNetEaseConfig,
    pub jd_sender: SenderJDCloudConfig,
    pub smser: Arc<SmsSender>,
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
        notify: Arc<Notify>,
        sender_task_size: Option<usize>,
        notify_task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let smser = Arc::new(SmsSender::new(
            app_core,
            redis.clone(),
            db.clone(),
            setting.clone(),
            fluent.clone(),
            logger,
            notify,
            sender_task_size,
            notify_task_size,
            task_timeout,
            is_check,
            None,
        ));

        let aliyun_sender =
            SenderAliYunConfig::new(setting.multiple.clone(), smser.tpl_config.clone());
        let hwyun_sender =
            SenderHwYunConfig::new(setting.multiple.clone(), smser.tpl_config.clone());
        let tenyun_sender =
            SenderTenYunConfig::new(setting.multiple.clone(), smser.tpl_config.clone());

        let cloopen_sender =
            SenderCloOpenConfig::new(setting.multiple.clone(), smser.tpl_config.clone());
        let netease_sender =
            SenderNetEaseConfig::new(setting.multiple.clone(), smser.tpl_config.clone());
        let jd_sender =
            SenderJDCloudConfig::new(setting.multiple.clone(), smser.tpl_config.clone());
        Self {
            smser,
            fluent,
            aliyun_sender,
            hwyun_sender,
            tenyun_sender,
            cloopen_sender,
            netease_sender,
            jd_sender,
        }
    }
    // 短信后台任务
    pub async fn task_sender(&self) -> Result<(), WebAppSmserError> {
        Ok(self
            .smser
            .task_sender(vec![
                Box::<AliYunSenderTask>::default(),
                Box::<HwYunSenderTask>::default(),
                Box::<TenyunSenderTask>::default(),
                Box::<NetEaseSenderTask>::default(),
                Box::<JDCloudSenderTask>::default(),
                Box::<CloOpenSenderTask>::default(),
            ])
            .await?)
    }
    // 短信发送状态查询任务
    pub async fn task_status_query(&self) -> Result<(), WebAppSmserError> {
        Ok(self
            .smser
            .task_status_query(vec![
                Box::<AliYunSendStatus>::default(),
                Box::<JDSendStatus>::default(),
                Box::<NetEaseSendStatus>::default(),
                Box::<TenYunSendStatus>::default(),
            ])
            .await?)
    }
    // 短信发送接口
    #[allow(clippy::too_many_arguments)]
    pub async fn app_send<'t>(
        &self,
        app: &AppsModel,
        tpl_type: &str,
        area: &'t str,
        mobile: &[&'t str],
        body: &HashMap<String, String>,
        send_time: &Option<u64>,
        max_try_num: &Option<u16>,
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, &'t str)>, WebAppSmserError> {
        let mb = mobile.iter().map(|e| (area, *e)).collect::<Vec<_>>();
        for tmp in mb.iter() {
            check_mobile(&self.fluent, tmp.0, tmp.1)
                .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        }
        let out = self
            .smser
            .send(
                Some(app.id),
                &mb,
                tpl_type,
                &json!(body).to_string(),
                send_time,
                &Some(app.user_id),
                max_try_num,
                env_data,
            )
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))?;
        Ok(out.1.into_iter().map(|e| (e.0, e.2)).collect::<Vec<_>>())
    }
    // APP 短信短信取消发送
    pub async fn app_send_cancel(
        &self,
        app: &AppsModel,
        id_data: &[u64],
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, bool)>, WebAppSmserError> {
        self.smser
            .cancal_from_message_id_vec(id_data, &app.user_id, env_data)
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))
    }
    // 通过消息取消发送
    pub async fn send_cancel(
        &self,
        body: &SenderSmsBodyModel,
        message: &[&SenderSmsMessageModel],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, bool)>, WebAppSmserError> {
        self.smser
            .cancal_from_message(body, message, &user_id, env_data)
            .await
            .map_err(|e| WebAppSmserError::System(e.to_string()))
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
