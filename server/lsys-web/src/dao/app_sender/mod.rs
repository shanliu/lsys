pub(crate) mod logger;
mod mailer;
mod smser;

use lsys_app::dao::AppNotify;
use lsys_app_sender::dao::{MailSenderConfig, MessageTpls, SmsSenderConfig};
use lsys_core::{AppCore, AppCoreError, IntoFluentMessage};
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::SettingDao;
pub use mailer::*;
pub use smser::*;

use sqlx::{MySql, Pool};
use std::sync::Arc;

use tracing::error;
pub struct AppSender {
    pub smser: Arc<SenderSmser>,
    pub mailer: Arc<SenderMailer>,
    pub tpl: Arc<MessageTpls>,
}

impl AppSender {
    pub async fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        notify: Arc<AppNotify>,
        setting: Arc<SettingDao>,
        change_logger: Arc<ChangeLoggerDao>,
    ) -> Result<AppSender, AppCoreError> {
        let tpl = Arc::new(MessageTpls::new(
            db.clone(),
            change_logger.clone(),
            app_core.create_tera().await?,
        ));
        let mailer = Arc::new(SenderMailer::new(
            app_core.clone(),
            redis.clone(),
            db.clone(),
            setting.clone(),
            change_logger.clone(),
            tpl.clone(),
            MailSenderConfig::default(),
        ));
        // 邮件发送任务

        tokio::spawn({
            let mail_task = mailer.clone();
            async move {
                if let Err(err) = mail_task.task_sender().await {
                    error!(
                        "mailer task error:{}",
                        err.to_fluent_message().default_format()
                    )
                }
            }
        });

        tokio::spawn({
            let mail_task_sendtime = mailer.clone();
            async move {
                mail_task_sendtime.task_sendtime_notify().await;
            }
        });

        tokio::spawn({
            let mail_wait = mailer.clone();
            async move { mail_wait.task_wait().await }
        });

        //启动回调任务
        let smser = Arc::new(SenderSmser::new(
            app_core.clone(),
            redis,
            db,
            setting,
            change_logger,
            notify,
            SmsSenderConfig::default(),
        ));
        //启动短信发送任务
        let sms_task_sender = smser.clone();
        tokio::spawn(async move {
            if let Err(err) = sms_task_sender.task_sender().await {
                error!(
                    "smser sender error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });
        //启动短信状态查询任务
        let sms_task_notify = smser.clone();
        tokio::spawn(async move {
            if let Err(err) = sms_task_notify.task_status_query().await {
                error!(
                    "smser notify error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });
        tokio::spawn({
            let sms_task_sendtime = smser.clone();
            async move {
                sms_task_sendtime.task_sendtime_notify().await;
            }
        });

        let sms_task_wait = smser.clone();
        tokio::spawn(async move { sms_task_wait.task_wait().await });

        Ok(AppSender { smser, mailer, tpl })
    }
}
