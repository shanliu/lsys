use lsys_app::model::AppModel;
use lsys_app_sender::dao::SenderError;
use lsys_core::{RequestEnv, ValidEmail, ValidParam, ValidParamCheck};
use serde_json::json;
use std::collections::HashMap;

use crate::common::JsonResult;

use super::SenderMailer;

impl SenderMailer {
    // 应用发送邮件接口
    #[allow(clippy::too_many_arguments)]
    pub async fn app_send<'t>(
        &self,
        app: &AppModel,
        tpl_id: &str,
        to: &[&'t str],
        body: &HashMap<&str, &str>,
        send_time: Option<u64>,
        reply: Option<&str>,
        max_try_num: Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(u64, &'t str)>> {
        let mut valid_param = ValidParam::default();
        for tmp in to.iter() {
            valid_param = valid_param.add(
                "to_email",
                tmp,
                &ValidParamCheck::default().add_rule(ValidEmail::default()),
            );
        }
        if let Some(cr) = reply {
            if !cr.is_empty() {
                valid_param = valid_param.add(
                    "reply_email",
                    &cr,
                    &ValidParamCheck::default().add_rule(ValidEmail::default()),
                );
            }
        }
        valid_param.check()?;
        let tos = to.to_vec();
        let res = self
            .mailer_dao
            .send(
                Some(app.id),
                &tos,
                tpl_id,
                &json!(body).to_string(),
                send_time,
                Some(app.user_id),
                reply,
                max_try_num,
                env_data,
            )
            .await
            .map(|e| e.1.into_iter().map(|e| (e.0, e.1)).collect::<Vec<_>>())?;
        Ok(res)
    }
    // APP 取消发送邮件
    pub async fn app_send_cancel(
        &self,
        app: &AppModel,
        snid_data: &[u64],
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(u64, bool, Option<SenderError>)>> {
        Ok(self
            .mailer_dao
            .cancal_from_message_snid_vec(snid_data, &app.user_id, env_data)
            .await?)
    }
}
