use std::{collections::HashMap, sync::Arc};

use lsys_app::model::AppsModel;
use lsys_core::{fluent_message, AppCore, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use lsys_notify::dao::Notify;
use lsys_sender::{
    dao::{
        AliYunSendStatus, AliYunSenderTask, CloOpenSenderTask, HwYunSenderTask, JDCloudSenderTask,
        JDSendStatus, NetEaseSendStatus, NetEaseSenderTask, SenderAliYunConfig,
        SenderCloOpenConfig, SenderError, SenderHwYunConfig, SenderJDCloudConfig,
        SenderNetEaseConfig, SenderResult, SenderTenYunConfig, SmsSender, TenYunSendStatus,
        TenyunSenderTask,
    },
    model::{SenderSmsBodyModel, SenderSmsMessageModel},
};
use lsys_setting::dao::Setting;
use lsys_user::dao::account::check_mobile;
use serde_json::json;
use sqlx::{MySql, Pool};

pub struct WebAppSmser {
    pub aliyun_sender: SenderAliYunConfig,
    pub hwyun_sender: SenderHwYunConfig,
    pub tenyun_sender: SenderTenYunConfig,
    pub cloopen_sender: SenderCloOpenConfig,
    pub netease_sender: SenderNetEaseConfig,
    pub jd_sender: SenderJDCloudConfig,
    pub smser: Arc<SmsSender>,
}

impl WebAppSmser {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
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
            logger,
            notify,
            sender_task_size,
            notify_task_size,
            task_timeout,
            is_check,
            None,
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
            aliyun_sender,
            hwyun_sender,
            tenyun_sender,
            cloopen_sender,
            netease_sender,
            jd_sender,
        }
    }
    pub async fn task_wait(&self) {
        self.smser.task_wait().await
    }
    // 短信后台任务
    pub async fn task_sender(&self) -> SenderResult<()> {
        self.smser
            .task_sender(vec![
                Box::<AliYunSenderTask>::default(),
                Box::<HwYunSenderTask>::default(),
                Box::<TenyunSenderTask>::default(),
                Box::<NetEaseSenderTask>::default(),
                Box::<JDCloudSenderTask>::default(),
                Box::<CloOpenSenderTask>::default(),
            ])
            .await
    }
    // 短信发送状态查询任务
    pub async fn task_status_query(&self) -> SenderResult<()> {
        self.smser
            .task_status_query(vec![
                Box::<AliYunSendStatus>::default(),
                Box::<JDSendStatus>::default(),
                Box::<NetEaseSendStatus>::default(),
                Box::<TenYunSendStatus>::default(),
            ])
            .await
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
        max_try_num: &Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, &'t str)>, SenderError> {
        let mb = mobile.iter().map(|e| (area, *e)).collect::<Vec<_>>();
        for tmp in mb.iter() {
            check_mobile(tmp.0, tmp.1).map_err(|e| {
                SenderError::System(fluent_message!("sms-send-check",{
                    "mobile":tmp.1,
                    "msg":e
                }))
            })?;
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
            .await?;
        Ok(out.1.into_iter().map(|e| (e.0, e.2)).collect::<Vec<_>>())
    }
    // APP 短信短信取消发送
    pub async fn app_send_cancel(
        &self,
        app: &AppsModel,
        snid_data: &[u64],
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, bool, Option<SenderError>)>, SenderError> {
        self.smser
            .cancal_from_message_snid_vec(snid_data, &app.user_id, env_data)
            .await
    }
    // 通过消息取消发送
    pub async fn send_cancel(
        &self,
        body: &SenderSmsBodyModel,
        message: &[&SenderSmsMessageModel],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> Result<Vec<(u64, bool, Option<SenderError>)>, SenderError> {
        self.smser
            .cancal_from_message(body, message, &user_id, env_data)
            .await
    }
    // 短信发送接口
    async fn send(
        &self,
        tpl_type: &str,
        area: &str,
        mobile: &str,
        body: &str,
        max_try_num: &Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> Result<u64, SenderError> {
        check_mobile(area, mobile).map_err(|e| {
            SenderError::System(fluent_message!("sms-send-check",{
                "mobile":mobile,
                "msg":e
            }))
        })?;
        let mut out = self
            .smser
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
            .await?;

        Ok(match out.1.pop() {
            Some(tmp1) => {
                tmp1.3.map_err(SenderError::System)?;
                tmp1.0
            }
            None => {
                return Err(SenderError::System(fluent_message!(
                    "mail-send-check",
                    "unkown error"
                )))
            }
        })
    }
    pub async fn send_valid_code(
        &self,
        area: &str,
        mobile: &str,
        code: &str,
        ttl: &usize,
        env_data: Option<&RequestEnv>,
    ) -> Result<u64, SenderError> {
        let mut context = HashMap::new();
        context.insert("code", code.to_owned());
        context.insert("time", ttl.to_string());
        self.send(
            "valid_code",
            area,
            mobile,
            &json!(context).to_string(),
            &Some(0),
            env_data,
        )
        .await
    }
}
