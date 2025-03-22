use std::sync::Arc;

use crate::{
    dao::{
        adapter::smser::sms_result_to_task, create_sender_client, SenderError, SenderExecError,
        SenderResult, SenderTaskExecutor, SenderTaskResult, SenderTplConfig, SmsSendNotifyParse,
        SmsTaskData, SmsTaskItem,
    },
    model::SenderTplConfigModel,
};
use async_trait::async_trait;

use lsys_core::{fluent_message, IntoFluentMessage, RequestEnv};
use lsys_lib_sms::{template_map_to_arr, CloOpenSms, SendError, SendNotifyError, SendNotifyItem};
use lsys_setting::{
    dao::{
        MultipleSetting, MultipleSettingData, SettingData, SettingDecode, SettingEncode,
        SettingError, SettingKey, SettingResult,
    },
    model::SettingModel,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;
//腾讯云 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct CloOpenConfig {
    pub account_sid: String,
    pub account_token: String,
    pub sms_app_id: String,
    pub branch_limit: u16,
    pub callback_key: String,
}

impl CloOpenConfig {
    pub fn hide_access_key(&self) -> String {
        let len = self.account_sid.chars().count();
        format!(
            "{}**{}",
            self.account_sid.chars().take(2).collect::<String>(),
            self.account_sid
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
    pub fn hide_access_secret(&self) -> String {
        let len = self.account_token.chars().count();
        format!(
            "{}**{}",
            self.account_token.chars().take(2).collect::<String>(),
            self.account_token
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
}

impl SettingKey for CloOpenConfig {
    fn key<'t>() -> &'t str {
        "col-sms-config"
    }
}
impl SettingDecode for CloOpenConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_str::<Self>(data).map_err(SettingError::SerdeJson)
    }
}

impl SettingEncode for CloOpenConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct CloOpenTplConfig {
    pub template_id: String,
    pub template_map: String,
}

//腾讯云发送短信配置
pub struct SenderCloOpenConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderCloOpenConfig {
    pub fn new(setting: Arc<MultipleSetting>, tpl_config: Arc<SenderTplConfig>) -> Self {
        Self {
            setting,
            tpl_config,
        }
    }
    //列出有效的jd_cloud短信配置
    pub async fn list_config(
        &self,
        config_ids: Option<&[u64]>,
    ) -> SenderResult<Vec<SettingData<CloOpenConfig>>> {
        let data = self
            .setting
            .list_data::<CloOpenConfig>(None, config_ids, None)
            .await?;
        Ok(data)
    }
    //删除指定的jd_cloud短信配置
    pub async fn del_config(
        &self,
        id: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .del::<CloOpenConfig>(None, id, user_id, None, env_data)
            .await?)
    }
    //编辑指定的jd_cloud短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        account_sid: &str,
        account_token: &str,
        sms_app_id: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        if branch_limit > CloOpenSms::branch_limit() {
            return Err(SenderError::System(
                fluent_message!("sms-config-branch-error",
                    {"max":CloOpenSms::branch_limit()}
                ),
            ));
        }
        Ok(self
            .setting
            .edit(
                None,
                id,
                &MultipleSettingData {
                    name,
                    data: &CloOpenConfig {
                        account_sid: account_sid.to_owned(),
                        account_token: account_token.to_owned(),
                        branch_limit,
                        sms_app_id: sms_app_id.to_owned(),
                        callback_key: callback_key.to_owned(),
                    },
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //添加短信配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_config(
        &self,
        name: &str,
        account_sid: &str,
        account_token: &str,
        sms_app_id: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .add(
                None,
                &MultipleSettingData {
                    name,
                    data: &CloOpenConfig {
                        account_sid: account_sid.to_owned(),
                        account_token: account_token.to_owned(),
                        sms_app_id: sms_app_id.to_owned(),
                        branch_limit,
                        callback_key: callback_key.to_owned(),
                    },
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //关联发送跟jd_cloud短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_id: &str,
        template_id: &str,
        template_map: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.setting.load::<CloOpenConfig>(None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_id,
                &CloOpenTplConfig {
                    template_id: template_id.to_owned(),
                    template_map: template_map.to_owned(),
                },
                user_id,
                add_user_id,
                env_data,
            )
            .await
    }
}

//腾讯云发送短信后台发送任务实现
#[derive(Clone, Default)]
pub struct CloOpenSenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData> for CloOpenSenderTask {
    fn setting_key(&self) -> String {
        CloOpenConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<CloOpenConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    CloOpenSms::branch_limit()
                } else {
                    e.branch_limit
                }
            })
            .unwrap_or(CloOpenSms::branch_limit())
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &SmsTaskItem,
        sms_data: &SmsTaskData,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> SenderTaskResult {
        let sub_setting =
            SettingData::<CloOpenConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let sub_tpl_config = serde_json::from_str::<CloOpenTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to tpl config fail[{}]:{}",
                    sub_setting.account_sid, e
                ))
            })?;

        debug!(
            "msgid:{} tpl_config_id:{} sms_app_id:{} tpl:{} var:{}",
            val.sms.id,
            tpl_config.id,
            sub_setting.sms_app_id,
            sub_tpl_config.template_id,
            val.sms.tpl_var
        );

        let mobile = sms_data
            .data
            .iter()
            .map(|e| e.mobile.as_str())
            .collect::<Vec<_>>();

        match CloOpenSms::branch_send(
            create_sender_client()?,
            &sub_setting.account_sid,
            &sub_setting.account_token,
            &sub_setting.sms_app_id,
            &sub_tpl_config.template_id,
            template_map_to_arr(&val.sms.tpl_var, &sub_tpl_config.template_map),
            &mobile,
        )
        .await
        {
            Ok(resp) => Ok(sms_result_to_task(&sms_data.data, &resp)),
            Err(err) => Err(match err {
                SendError::Next(e) => SenderExecError::Next(e),
                SendError::Finish(e) => SenderExecError::Finish(e),
            }),
        }
    }
}

pub struct CloOpenNotify<'t> {
    callback_key: &'t str,
    notify_data: &'t str,
}
impl<'t> CloOpenNotify<'t> {
    pub fn new(callback_key: &'t str, notify_data: &'t str) -> CloOpenNotify<'t> {
        Self {
            callback_key,
            notify_data,
        }
    }
}

#[async_trait]
impl SmsSendNotifyParse for CloOpenNotify<'_> {
    type T = CloOpenConfig;
    fn notify_items(
        &self,
        config: &SettingData<CloOpenConfig>,
    ) -> Result<Vec<SendNotifyItem>, SendNotifyError> {
        if !config.callback_key.is_empty() && config.callback_key.as_str() != self.callback_key {
            return Err(SendNotifyError::Sign(format!(
                "callback key is match :{}",
                self.callback_key
            )));
        }
        CloOpenSms::send_notify_parse(self.notify_data)
    }
    fn output(res: &Result<(), String>) -> (u16, String) {
        let (code, msg) = CloOpenSms::send_notify_output(res);
        (code.as_u16(), msg)
    }
}
