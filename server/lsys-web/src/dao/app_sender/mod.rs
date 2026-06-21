pub(crate) mod logger;
mod mailer;
mod smser;

use crate::dao::WebResult;
use lsys_app::dao::AppNotify;
use lsys_app_sender::dao::{MailSenderConfig, MessageTpls, SmsSenderConfig};
use lsys_core::app_core::AppCore;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::secret::FieldEncryptor;
use lsys_core::task_lifecycle::TaskNode;
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
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        notify: Arc<AppNotify>,
        setting: Arc<SettingDao>,
        change_logger: Arc<ChangeLoggerDao>,
        smtp_encryptor: Arc<FieldEncryptor>,
        task_node: Arc<TaskNode>,
    ) -> WebResult<AppSender> {
        let tpl = Arc::new(MessageTpls::new(
            db.clone(),
            change_logger.clone(),
            lsys_core::app_core::create_tera(&app_core).await?,
        ));
        let mailer = Arc::new(SenderMailer::new(
            app_core.clone(),
            redis.clone(),
            db.clone(),
            setting.clone(),
            change_logger.clone(),
            tpl.clone(),
            MailSenderConfig::default(),
            smtp_encryptor,
        ));
        // 邮件发送任务
        let mailer_node = task_node.child("app-sender-mailer");

        let mailer_task = mailer.clone();
        mailer_node.spawn(move |token| async move {
            if let Err(err) = mailer_task.task_sender(token).await {
                error!(
                    "mailer task error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });

        let mailer_sendtime = mailer.clone();
        mailer_node.spawn(move |token| async move {
            mailer_sendtime.task_sendtime_notify(token).await;
        });

        let mailer_wait = mailer.clone();
        mailer_node.spawn(move |token| async move {
            mailer_wait.task_wait(token).await
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
        let smser_node = task_node.child("app-sender-smser");

        let smser_sender = smser.clone();
        smser_node.spawn(move |token| async move {
            if let Err(err) = smser_sender.task_sender(token).await {
                error!(
                    "smser sender error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });
        //启动短信状态查询任务
        let smser_status = smser.clone();
        smser_node.spawn(move |token| async move {
            if let Err(err) = smser_status.task_status_query(token).await {
                error!(
                    "smser notify error:{}",
                    err.to_fluent_message().default_format()
                )
            }
        });
        let smser_sendtime = smser.clone();
        smser_node.spawn(move |token| async move {
            smser_sendtime.task_sendtime_notify(token).await;
        });

        let smser_wait = smser.clone();
        smser_node.spawn(move |token| async move { smser_wait.task_wait(token).await });

        Ok(AppSender { smser, mailer, tpl })
    }
}
