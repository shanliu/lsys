mod app;

use lsys_access::dao::SessionBody;
use lsys_app_sender::{
    dao::{
        MailSenderDao, MessageTpls, SenderError, SenderResult, SenderSmtpConfig, SmtpSenderTask,
    },
    model::{SenderMailBodyModel, SenderMailMessageModel},
};
use lsys_core::{fluent_message, AppCore, RequestEnv};
use lsys_logger::dao::ChangeLoggerDao;
use lsys_setting::dao::SettingDao;
use sqlx::{MySql, Pool};
use std::sync::Arc;
use tera::Context;

use crate::common::{JsonError, JsonResult};

use super::logger::MessageView;

pub struct SenderMailer {
    tpls: Arc<MessageTpls>,
    logger: Arc<ChangeLoggerDao>,
    pub mailer_dao: Arc<MailSenderDao>,
    pub smtp_sender: SenderSmtpConfig,
}

impl SenderMailer {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        app_core: Arc<AppCore>,
        redis: deadpool_redis::Pool,
        db: Pool<MySql>,
        setting: Arc<SettingDao>,
        logger: Arc<ChangeLoggerDao>,
        tpls: Arc<MessageTpls>,
        task_size: Option<usize>,
        task_timeout: usize,
        is_check: bool,
    ) -> Self {
        let mailer_dao = Arc::new(MailSenderDao::new(
            app_core.clone(),
            redis,
            db,
            setting.clone(),
            logger.clone(),
            task_size,
            task_timeout,
            is_check,
            None,
        ));
        let smtp_sender =
            SenderSmtpConfig::new(setting.multiple.clone(), mailer_dao.tpl_config.clone());
        Self {
            mailer_dao,
            smtp_sender,
            logger,
            tpls,
        }
    }
    pub async fn send_valid_code(
        &self,
        tpl_key: &str,
        to: &str,
        code: &str,
        ttl: &usize,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        let mut context = Context::new();
        context.insert("code", code);
        context.insert("ttl", ttl);
        self.send(
            tpl_key,
            to,
            &context.into_json().to_string(),
            Some(0),
            env_data,
        )
        .await
        .map(|_| ())
    }
    // 发送接口
    async fn send(
        &self,
        tpl_key: &str,
        to: &str,
        body: &str,
        max_try_num: Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<u64> {
        let mut out = self
            .mailer_dao
            .send(
                None,
                &[to],
                tpl_key,
                body,
                None,
                None,
                None,
                max_try_num,
                env_data,
            )
            .await?;

        Ok(match out.1.pop() {
            Some(tmp1) => {
                tmp1.2.map_err(SenderError::System)?;
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
    // 后台任务
    pub async fn task_sender(&self) -> SenderResult<()> {
        let task = SmtpSenderTask::new(self.tpls.clone());
        self.mailer_dao.task_sender(vec![Box::new(task)]).await
    }
    // 后台任务
    pub async fn task_wait(&self) {
        self.mailer_dao.task_wait().await
    }
    // 取消发送接口
    pub async fn send_cancel(
        &self,
        body: &SenderMailBodyModel,
        message: &[&SenderMailMessageModel],
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(u64, bool, Option<SenderError>)>> {
        Ok(self
            .mailer_dao
            .cancal_from_message(body, message, user_id, env_data)
            .await?)
    }
    //记录邮件查看日志
    pub async fn mailer_message_body(
        &self,
        message: &SenderMailMessageModel,
        body: &SenderMailBodyModel,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        self.logger
            .add(
                &MessageView {
                    msg_type: "mail",
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
