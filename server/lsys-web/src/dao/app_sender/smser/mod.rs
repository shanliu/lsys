mod app;
pub use app::*;
use lsys_access::dao::SessionBody;
use std::{collections::HashMap, sync::Arc};

use lsys_app_notify::dao::NotifyDao;
use lsys_app_sender::{
    dao::{
        AliYunSendStatus, AliYunSenderTask, CloOpenSenderTask, HwYunSenderTask, JDCloudSenderTask,
        JDSendStatus, NetEaseSendStatus, NetEaseSenderTask, SenderAliYunConfig,
        SenderCloOpenConfig, SenderError, SenderHwYunConfig, SenderJDCloudConfig,
        SenderNetEaseConfig, SenderResult, SenderTenYunConfig, SmsSenderDao, TenYunSendStatus,
        TenyunSenderTask,
    },
    model::{SenderSmsBodyModel, SenderSmsMessageModel},
};
use lsys_core::{fluent_message, AppCore, RequestEnv};
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::SettingDao;
use lsys_user::dao::check_mobile;
use serde_json::json;
use sqlx::{MySql, Pool};

use crate::common::{JsonError, JsonResult};

use super::logger::MessageView;

pub struct SenderSmser {
    pub aliyun_sender: SenderAliYunConfig,
    pub hwyun_sender: SenderHwYunConfig,
    pub tenyun_sender: SenderTenYunConfig,
    pub cloopen_sender: SenderCloOpenConfig,
    pub netease_sender: SenderNetEaseConfig,
    pub jd_sender: SenderJDCloudConfig,
    pub smser_dao: Arc<SmsSenderDao>,
    logger: Arc<ChangeLoggerDao>,
}

impl SenderSmser {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        setting: Arc<SettingDao>,
        logger: Arc<ChangeLoggerDao>,
        notify: Arc<NotifyDao>,
        sender_task_size: Option<usize>,
        notify_task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let smser_dao = Arc::new(SmsSenderDao::new(
            app_core,
            redis.clone(),
            db.clone(),
            setting.clone(),
            logger.clone(),
            notify.clone(),
            sender_task_size,
            notify_task_size,
            task_timeout,
            is_check,
            None,
            None,
        ));

        let aliyun_sender =
            SenderAliYunConfig::new(setting.multiple.clone(), smser_dao.tpl_config.clone());
        let hwyun_sender =
            SenderHwYunConfig::new(setting.multiple.clone(), smser_dao.tpl_config.clone());
        let tenyun_sender =
            SenderTenYunConfig::new(setting.multiple.clone(), smser_dao.tpl_config.clone());

        let cloopen_sender =
            SenderCloOpenConfig::new(setting.multiple.clone(), smser_dao.tpl_config.clone());
        let netease_sender =
            SenderNetEaseConfig::new(setting.multiple.clone(), smser_dao.tpl_config.clone());
        let jd_sender =
            SenderJDCloudConfig::new(setting.multiple.clone(), smser_dao.tpl_config.clone());

        Self {
            smser_dao,
            aliyun_sender,
            hwyun_sender,
            tenyun_sender,
            cloopen_sender,
            netease_sender,
            jd_sender,
            logger,
        }
    }
    pub async fn task_wait(&self) {
        self.smser_dao.task_wait().await
    }
    // 短信后台任务
    pub async fn task_sender(&self) -> SenderResult<()> {
        self.smser_dao
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
        self.smser_dao
            .task_status_query(vec![
                Box::<AliYunSendStatus>::default(),
                Box::<JDSendStatus>::default(),
                Box::<NetEaseSendStatus>::default(),
                Box::<TenYunSendStatus>::default(),
            ])
            .await
    }
    // 通过消息取消发送
    pub async fn send_cancel(
        &self,
        body: &SenderSmsBodyModel,
        message: &[&SenderSmsMessageModel],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(u64, bool, Option<SenderError>)>> {
        Ok(self
            .smser_dao
            .cancal_from_message(body, message, &user_id, env_data)
            .await?)
    }
    // 短信发送接口
    async fn send(
        &self,
        tpl_type: &str,
        area: &str,
        mobile: &str,
        body: &str,
        max_try_num: Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        check_mobile(area, mobile)?;
        let mut out = self
            .smser_dao
            .send(
                None,
                &[(area, mobile)],
                tpl_type,
                body,
                None,
                None,
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
                return Err(JsonError::Message(fluent_message!(
                    "mail-send-check",
                    "unkown error"
                )))
            }
        })
    }
    //发送验证码
    pub async fn send_valid_code(
        &self,
        area: &str,
        mobile: &str,
        code: &str,
        ttl: &usize,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let mut context = HashMap::new();
        context.insert("code", code.to_owned());
        context.insert("time", ttl.to_string());
        self.send(
            "valid_code",
            area,
            mobile,
            &json!(context).to_string(),
            Some(0),
            env_data,
        )
        .await
    }
    //记录短信查看日志
    pub async fn smser_message_body(
        &self,
        message: &SenderSmsMessageModel,
        body: &SenderSmsBodyModel,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        self.logger
            .add(
                &MessageView {
                    msg_type: "sms",
                    body_id: body.id,
                    user_id: body.user_id,
                },
                Some(message.id),
                Some(session_body.user_id()),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}
