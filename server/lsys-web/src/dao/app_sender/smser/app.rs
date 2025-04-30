use std::collections::HashMap;

use lsys_app::model::AppModel;
use lsys_app_sender::dao::SenderError;
use lsys_core::{RequestEnv, ValidMobile, ValidParam, ValidParamCheck};
use serde_json::json;

use crate::common::JsonResult;

use super::SenderSmser;

impl SenderSmser {
    // app 短信发送接口
    #[allow(clippy::too_many_arguments)]
    pub async fn app_send<'t>(
        &self,
        app: &AppModel,
        tpl_type: &str,
        area: &'t str,
        mobile: &[&'t str],
        body: &HashMap<&str, &str>,
        send_time: Option<u64>,
        max_try_num: Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(u64, &'t str)>> {
        let mut valid_param = ValidParam::default();
        for tmp in mobile.iter() {
            valid_param = valid_param.add(
                "to_mobile",
                &format!("{}{}", area, tmp),
                &ValidParamCheck::default().add_rule(ValidMobile::default()),
            )
        }
        valid_param.check()?;

        let out = self
            .smser_dao
            .send(
                Some(app.id),
                &mobile.iter().map(|e| (area, *e)).collect::<Vec<_>>(),
                tpl_type,
                &json!(body).to_string(),
                send_time,
                Some(app.user_id),
                max_try_num,
                env_data,
            )
            .await?;
        Ok(out.1.into_iter().map(|e| (e.0, e.2)).collect::<Vec<_>>())
    }
    // APP 短信短信取消发送
    pub async fn app_send_cancel(
        &self,
        app: &AppModel,
        snid_data: &[u64],
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<Vec<(u64, bool, Option<SenderError>)>> {
        Ok(self
            .smser_dao
            .cancal_from_message_snid_vec(snid_data, &app.user_id, env_data)
            .await?)
    }
}
