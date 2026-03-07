use std::sync::Arc;

use crate::{
    dao::{
        adapter::smser::sms_result_to_task, create_sender_client, SenderExecError, SenderResult,
        SenderTaskExecutor, SenderTaskResult, SenderTplConfig, SmsSendNotifyParse, SmsTaskData,
        SmsTaskItem,
    },
    model::SenderTplConfigModel,
};
use async_trait::async_trait;

use lsys_core::db::OffsetPageParam;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::utils::RequestEnv;
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
use lsys_lib_sms::{EmaySms, SendError, SendNotifyError, SendNotifyItem};
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
//亿美软通 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct EmayConfig {
    pub host: String,
    pub app_id: String,
    pub secret_key: String,
    pub branch_limit: u16,
    pub callback_key: String,
}

impl EmayConfig {
    pub fn hide_app_id(&self) -> String {
        let len = self.app_id.chars().count();
        format!(
            "{}**{}",
            self.app_id.chars().take(2).collect::<String>(),
            self.app_id
                .chars()
                .skip(if len > 2 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
    pub fn hide_secret_key(&self) -> String {
        let len = self.secret_key.chars().count();
        format!(
            "{}**{}",
            self.secret_key.chars().take(2).collect::<String>(),
            self.secret_key
                .chars()
                .skip(if len > 2 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
}

impl SettingKey for EmayConfig {
    fn key<'t>() -> &'t str {
        "emay-sms-config"
    }
}
impl SettingDecode for EmayConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_str::<Self>(data).map_err(SettingError::SerdeJson)
    }
}

impl SettingEncode for EmayConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct EmayTplConfig {
    pub extended_code: String,
}

//亿美软通发送短信配置
pub struct SenderEmayConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderEmayConfig {
    pub fn new(setting: Arc<MultipleSetting>, tpl_config: Arc<SenderTplConfig>) -> Self {
        Self {
            setting,
            tpl_config,
        }
    }
    //列出有效的emay短信配置
    pub async fn list_config(
        &self,
        config_ids: Option<&[u64]>,
    ) -> SenderResult<Vec<SettingData<EmayConfig>>> {
        let data = self
            .setting
            .list_data::<EmayConfig>(None, config_ids, &OffsetPageParam::new(None))
            .await?;
        Ok(data)
    }
    //删除指定的emay短信配置
    pub async fn del_config(
        &self,
        id: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.tpl_config.check_setting_id_used(id).await?;
        Ok(self
            .setting
            .del::<EmayConfig>(None, id, user_id, None, env_data)
            .await?)
    }

    #[allow(clippy::too_many_arguments)]
    async fn edit_config_param_valid(
        &self,
        id: u64,
        name: &str,
        host: &str,
        app_id: &str,
        secret_key: &str,
        branch_limit: u16,
        callback_key: &str,
    ) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("config_id"),
                &id,
                &ValidParamCheck::default().add_rule(ValidNumber::id()),
            )
            .add(
                valid_key!("config_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 64)),
            )
            .add(
                valid_key!("host"),
                &host,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 256)),
            )
            .add(
                valid_key!("app_id"),
                &app_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("secret_key"),
                &secret_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("callback_key"),
                &callback_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(6, 32)),
            )
            .add(
                valid_key!("branch_limit"),
                &branch_limit,
                &ValidParamCheck::default()
                    .add_rule(ValidNumber::range(1, EmaySms::branch_limit())),
            )
            .check()?;
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        host: &str,
        app_id: &str,
        secret_key: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.edit_config_param_valid(
            id,
            name,
            host,
            app_id,
            secret_key,
            branch_limit,
            callback_key,
        )
        .await?;
        Ok(self
            .setting
            .edit(
                None,
                id,
                &MultipleSettingData {
                    name,
                    data: &EmayConfig {
                        host: host.to_owned(),
                        app_id: app_id.to_owned(),
                        secret_key: secret_key.to_owned(),
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
    #[allow(clippy::too_many_arguments)]
    async fn add_config_param_valid(
        &self,
        name: &str,
        host: &str,
        app_id: &str,
        secret_key: &str,
        branch_limit: u16,
        callback_key: &str,
    ) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("config_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 64)),
            )
            .add(
                valid_key!("host"),
                &host,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 256)),
            )
            .add(
                valid_key!("app_id"),
                &app_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("secret_key"),
                &secret_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("callback_key"),
                &callback_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(6, 32)),
            )
            .add(
                valid_key!("branch_limit"),
                &branch_limit,
                &ValidParamCheck::default()
                    .add_rule(ValidNumber::range(1, EmaySms::branch_limit())),
            )
            .check()?;
        Ok(())
    }
    //添加短信配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_config(
        &self,
        name: &str,
        host: &str,
        app_id: &str,
        secret_key: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_config_param_valid(name, host, app_id, secret_key, branch_limit, callback_key)
            .await?;
        Ok(self
            .setting
            .add(
                None,
                &MultipleSettingData {
                    name,
                    data: &EmayConfig {
                        host: host.to_owned(),
                        app_id: app_id.to_owned(),
                        secret_key: secret_key.to_owned(),
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
    async fn add_app_config_param_valid(&self, extended_code: &str) -> SenderResult<()> {
        let mut valid_param = ValidParam::default();
        valid_param.add(
            valid_key!("extended_code"),
            &extended_code,
            &ValidParamCheck::default().add_rule(ValidStrlen::range(0, 128)),
        );
        valid_param.check()?;
        Ok(())
    }
    //关联发送跟emay短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_key: &str,
        extended_code: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_app_config_param_valid(extended_code).await?;
        self.setting.load::<EmayConfig>(None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_key,
                &EmayTplConfig {
                    extended_code: extended_code.to_owned(),
                },
                user_id,
                add_user_id,
                env_data,
            )
            .await
    }
}

//亿美软通发送短信后台发送任务实现
#[derive(Clone, Default)]
pub struct EmaySenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData> for EmaySenderTask {
    fn setting_key(&self) -> String {
        EmayConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<EmayConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    EmaySms::branch_limit()
                } else {
                    e.branch_limit
                }
            })
            .unwrap_or(EmaySms::branch_limit())
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &SmsTaskItem,
        sms_data: &SmsTaskData,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> SenderTaskResult {
        let emay_setting =
            SettingData::<EmayConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to emay setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let emay_tpl_config = serde_json::from_str::<EmayTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to emay tpl config fail[{}]:{}",
                    emay_setting.app_id, e
                ))
            })?;

        debug!(
            "msgid:{} tpl_config_id:{} app_id:{} var:{}",
            val.sms.id, tpl_config.id, emay_setting.app_id, val.sms.tpl_var
        );

        let mobile = sms_data
            .data
            .iter()
            .map(|e| e.mobile.as_str())
            .collect::<Vec<_>>();

        let custom_sms_ids: Vec<String> = sms_data.data.iter().map(|e| e.id.to_string()).collect();

        match EmaySms::branch_send_custom(
            create_sender_client()?,
            &emay_setting.host,
            &emay_setting.app_id,
            &emay_setting.secret_key,
            &val.sms.tpl_var,
            &mobile,
            Some(custom_sms_ids),
            &emay_tpl_config.extended_code,
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

pub struct EmayNotify<'t> {
    callback_key: &'t str,
    notify_data: &'t str,
}
impl<'t> EmayNotify<'t> {
    pub fn new(callback_key: &'t str, notify_data: &'t str) -> EmayNotify<'t> {
        Self {
            callback_key,
            notify_data,
        }
    }
}

impl SmsSendNotifyParse for EmayNotify<'_> {
    type T = EmayConfig;
    fn notify_items(
        &self,
        config: &SettingData<EmayConfig>,
    ) -> Result<Vec<SendNotifyItem>, SendNotifyError> {
        if !config.callback_key.is_empty() && config.callback_key.as_str() != self.callback_key {
            return Err(SendNotifyError::Sign(format!(
                "callback key is match :{}",
                self.callback_key
            )));
        }
        EmaySms::send_notify_parse(self.notify_data)
    }
    fn output(res: &Result<(), String>) -> (u16, String) {
        (200, EmaySms::send_notify_output(res))
    }
}
