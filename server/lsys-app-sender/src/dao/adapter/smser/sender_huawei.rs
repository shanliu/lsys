use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::{
        adapter::smser::sms_result_to_task, create_sender_client, SenderExecError, SenderResult,
        SenderTaskExecutor, SenderTaskResult, SenderTplConfig, SmsSendNotifyParse, SmsTaskData,
        SmsTaskItem,
    },
    model::SenderTplConfigModel,
};
use async_trait::async_trait;
use lsys_core::{
    valid_key, IntoFluentMessage, RequestEnv, ValidNumber, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen, ValidUrl,
};
use lsys_setting::{
    dao::{
        MultipleSetting, MultipleSettingData, SettingData, SettingDecode, SettingEncode,
        SettingError, SettingKey, SettingResult,
    },
    model::SettingModel,
};

use lsys_lib_sms::{template_map_to_arr, HwSms, SendError, SendNotifyError, SendNotifyItem};
use serde::{Deserialize, Serialize};
use serde_json::json;

use tracing::debug;

//华为 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct HwYunConfig {
    pub app_key: String,
    pub app_secret: String,
    pub branch_limit: u16,
    pub callback_key: String,
    pub url: String,
}

impl HwYunConfig {
    pub fn hide_access_key(&self) -> String {
        let len = self.app_key.chars().count();
        format!(
            "{}**{}",
            self.app_key.chars().take(2).collect::<String>(),
            self.app_key
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
    pub fn hide_access_secret(&self) -> String {
        let len = self.app_secret.chars().count();
        format!(
            "{}**{}",
            self.app_secret.chars().take(2).collect::<String>(),
            self.app_secret
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
}

impl SettingKey for HwYunConfig {
    fn key<'t>() -> &'t str {
        "hwyun-sms-config"
    }
}
impl SettingDecode for HwYunConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_str::<Self>(data).map_err(SettingError::SerdeJson)
    }
}

impl SettingEncode for HwYunConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HwYunTplConfig {
    pub signature: String,
    pub sender: String,
    pub template_id: String,
    pub template_map: String,
}

//阿里云发送短信配置
pub struct SenderHwYunConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderHwYunConfig {
    pub fn new(setting: Arc<MultipleSetting>, tpl_config: Arc<SenderTplConfig>) -> Self {
        Self {
            setting,
            tpl_config,
        }
    }
    //列出有效的huawei短信配置
    pub async fn list_config(
        &self,
        config_ids: Option<&[u64]>,
    ) -> SenderResult<Vec<SettingData<HwYunConfig>>> {
        let data = self
            .setting
            .list_data::<HwYunConfig>(None, config_ids, None)
            .await?;
        Ok(data)
    }
    //删除指定的huawei短信配置
    pub async fn del_config(
        &self,
        id: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.tpl_config.check_setting_id_used(id).await?;
        Ok(self
            .setting
            .del::<HwYunConfig>(None, id, user_id, None, env_data)
            .await?)
    }

    #[allow(clippy::too_many_arguments)]
    async fn edit_config_param_valid(
        &self,
        id: u64,
        name: &str,
        url: &str,
        app_key: &str,
        app_secret: &str,
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
                valid_key!("hw_url"),
                &url,
                &ValidParamCheck::default()
                    .add_rule(ValidUrl::default())
                    .add_rule(ValidStrlen::max(255)),
            )
            .add(
                valid_key!("hw_app_key"),
                &app_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("hw_app_secret"),
                &app_secret,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
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
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, HwSms::branch_limit())),
            )
            .check()?;
        Ok(())
    }

    //编辑指定的huawei短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        url: &str,
        app_key: &str,
        app_secret: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.edit_config_param_valid(
            id,
            name,
            url,
            app_key,
            app_secret,
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
                    data: &HwYunConfig {
                        url: url.to_owned(),
                        app_key: app_key.to_owned(),
                        app_secret: app_secret.to_owned(),
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
        url: &str,
        app_key: &str,
        app_secret: &str,
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
                valid_key!("hw_url"),
                &url,
                &ValidParamCheck::default()
                    .add_rule(ValidUrl::default())
                    .add_rule(ValidStrlen::max(255)),
            )
            .add(
                valid_key!("hw_app_key"),
                &app_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("hw_app_secret"),
                &app_secret,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
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
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, HwSms::branch_limit())),
            )
            .check()?;

        Ok(())
    }
    //添加huawei短信配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_config(
        &self,
        name: &str,
        url: &str,
        app_key: &str,
        app_secret: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_config_param_valid(name, url, app_key, app_secret, branch_limit, callback_key)
            .await?;
        Ok(self
            .setting
            .add(
                None,
                &MultipleSettingData {
                    name,
                    data: &HwYunConfig {
                        url: url.to_owned(),
                        app_key: app_key.to_owned(),
                        app_secret: app_secret.to_owned(),
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
    async fn add_app_config_param_valid(
        &self,

        signature: &str,
        sender: &str,
        template_id: &str,
        template_map: &str,
    ) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("template_id"),
                &template_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("template_map"),
                &template_map,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 2000)),
            )
            .add(
                valid_key!("hw_signature"),
                &signature,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 64)),
            )
            .add(
                valid_key!("hw_sender"),
                &sender,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 64)),
            )
            .check()?;
        Ok(())
    }
    //关联发送跟huawei短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_key: &str,
        signature: &str,
        sender: &str,
        template_id: &str,
        template_map: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_app_config_param_valid(signature, sender, template_id, template_map)
            .await?;
        self.setting.load::<HwYunConfig>(None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_key,
                &HwYunTplConfig {
                    template_map: template_map.to_owned(),
                    signature: signature.to_owned(),
                    sender: sender.to_owned(),
                    template_id: template_id.to_owned(),
                },
                user_id,
                add_user_id,
                env_data,
            )
            .await
    }
}

//阿里云发送短信后台发送任务实现
#[derive(Clone, Default)]
pub struct HwYunSenderTask {
    callback_url: String,
}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData> for HwYunSenderTask {
    fn setting_key(&self) -> String {
        HwYunConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<HwYunConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    HwSms::branch_limit()
                } else {
                    e.branch_limit
                }
            })
            .unwrap_or(HwSms::branch_limit())
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
            SettingData::<HwYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to huawei setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let sub_tpl_config = serde_json::from_str::<HwYunTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to huawei tpl config fail[{}]:{}",
                    sub_setting.app_key, e
                ))
            })?;
        debug!(
            "msgid:{}   tpl_config_id:{} access_id:{} tpl:{} var:{}",
            val.sms.id,
            tpl_config.id,
            sub_setting.app_key,
            sub_tpl_config.template_id,
            val.sms.tpl_var
        );
        let mobile = sms_data
            .data
            .iter()
            .map(|e| e.mobile.as_str())
            .collect::<Vec<_>>();

        match HwSms::branch_send(
            create_sender_client()?,
            &sub_setting.url,
            &sub_setting.app_key,
            &sub_setting.app_secret,
            &sub_tpl_config.signature,
            &sub_tpl_config.sender,
            &sub_tpl_config.template_id,
            template_map_to_arr(&val.sms.tpl_var, &sub_tpl_config.template_map),
            &mobile,
            &self.callback_url,
            "",
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

pub struct HwYunNotify<'t> {
    callback_key: &'t str,
    notify_data: &'t HashMap<String, String>,
}
impl<'t> HwYunNotify<'t> {
    pub fn new(callback_key: &'t str, notify_data: &'t HashMap<String, String>) -> HwYunNotify<'t> {
        Self {
            callback_key,
            notify_data,
        }
    }
}

#[async_trait]
impl SmsSendNotifyParse for HwYunNotify<'_> {
    type T = HwYunConfig;
    fn notify_items(
        &self,
        config: &SettingData<HwYunConfig>,
    ) -> Result<Vec<SendNotifyItem>, SendNotifyError> {
        if !config.callback_key.is_empty() && config.callback_key.as_str() != self.callback_key {
            return Err(SendNotifyError::Sign(format!(
                "callback key is match :{}",
                self.callback_key
            )));
        }
        HwSms::send_notify_parse(self.notify_data)
    }
    fn output(res: &Result<(), String>) -> (u16, String) {
        (200, HwSms::send_notify_output(res))
    }
}
