use std::sync::Arc;

use crate::{
    dao::{
        adapter::smser::sms_result_to_task, create_sender_client, SenderExecError, SenderResult,
        SenderTaskExecutor, SenderTaskResult, SenderTplConfig, SmsSendNotifyParse, SmsTaskData,
        SmsTaskItem,
    },
    model::{SenderSmsMessageModel, SenderTplConfigModel},
};
use async_trait::async_trait;

use chrono::DateTime;
use lsys_core::{
    valid_key, IntoFluentMessage, RequestEnv, ValidNumber, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use lsys_lib_sms::{
    template_map_to_arr, SendDetailItem, SendError, SendNotifyError, SendNotifyItem, TenSms,
};
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
pub struct TenYunConfig {
    pub region: String,
    pub secret_id: String,
    pub secret_key: String,
    pub sms_app_id: String,
    pub branch_limit: u16,
    pub callback_key: String,
}

impl TenYunConfig {
    pub fn hide_secret_id(&self) -> String {
        let len = self.secret_id.chars().count();
        format!(
            "{}**{}",
            self.secret_id.chars().take(2).collect::<String>(),
            self.secret_id
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
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
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
}

impl SettingKey for TenYunConfig {
    fn key<'t>() -> &'t str {
        "tenyun-sms-config"
    }
}
impl SettingDecode for TenYunConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_str::<Self>(data).map_err(SettingError::SerdeJson)
    }
}

impl SettingEncode for TenYunConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TenYunTplConfig {
    pub template_id: String,
    pub sign_name: String,
    pub template_map: String,
}

//腾讯云发送短信配置
pub struct SenderTenYunConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderTenYunConfig {
    pub fn new(setting: Arc<MultipleSetting>, tpl_config: Arc<SenderTplConfig>) -> Self {
        Self {
            setting,
            tpl_config,
        }
    }
    //列出有效的tenyun短信配置
    pub async fn list_config(
        &self,
        config_ids: Option<&[u64]>,
    ) -> SenderResult<Vec<SettingData<TenYunConfig>>> {
        let data = self
            .setting
            .list_data::<TenYunConfig>(None, config_ids, None)
            .await?;
        Ok(data)
    }
    //删除指定的tenyun短信配置
    pub async fn del_config(
        &self,
        id: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.tpl_config.check_setting_id_used(id).await?;
        Ok(self
            .setting
            .del::<TenYunConfig>(None, id, user_id, None, env_data)
            .await?)
    }
    #[allow(clippy::too_many_arguments)]
    async fn edit_config_param_valid(
        &self,
        id: u64,
        name: &str,
        region: &str,
        secret_id: &str,
        secret_key: &str,
        sms_app_id: &str,
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
                valid_key!("region"),
                &region,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("secret_id"),
                &secret_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("secret_key"),
                &secret_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("sms_app_id"),
                &sms_app_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("sms_app_id"),
                &sms_app_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("callback_key"),
                &callback_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(6, 128)),
            )
            .add(
                valid_key!("branch_limit"),
                &branch_limit,
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, TenSms::branch_limit())),
            )
            .check()?;
        Ok(())
    }
    //编辑指定的tenyun短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        region: &str,
        secret_id: &str,
        secret_key: &str,
        sms_app_id: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.edit_config_param_valid(
            id,
            name,
            region,
            secret_id,
            secret_key,
            sms_app_id,
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
                    data: &TenYunConfig {
                        region: region.to_owned(),
                        sms_app_id: sms_app_id.to_owned(),
                        branch_limit,
                        secret_id: secret_id.to_owned(),
                        secret_key: secret_key.to_owned(),
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
        region: &str,
        secret_id: &str,
        secret_key: &str,
        sms_app_id: &str,
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
                valid_key!("region"),
                &region,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("secret_id"),
                &secret_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("secret_key"),
                &secret_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("sms_app_id"),
                &sms_app_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("sms_app_id"),
                &sms_app_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("callback_key"),
                &callback_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(6, 128)),
            )
            .add(
                valid_key!("branch_limit"),
                &branch_limit,
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, TenSms::branch_limit())),
            )
            .check()?;
        Ok(())
    }
    //添加tenyun短信配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_config(
        &self,
        name: &str,
        region: &str,
        secret_id: &str,
        secret_key: &str,
        sms_app_id: &str,
        branch_limit: u16,
        callback_key: &str,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_config_param_valid(
            name,
            region,
            secret_id,
            secret_key,
            sms_app_id,
            branch_limit,
            callback_key,
        )
        .await?;
        Ok(self
            .setting
            .add(
                None,
                &MultipleSettingData {
                    name,
                    data: &TenYunConfig {
                        region: region.to_owned(),
                        sms_app_id: sms_app_id.to_owned(),
                        branch_limit,
                        secret_id: secret_id.to_owned(),
                        secret_key: secret_key.to_owned(),
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

        sign_name: &str,
        template_id: &str,
        template_map: &str,
    ) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("sign_name"),
                &sign_name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
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
            .check()?;
        Ok(())
    }
    //关联发送跟tenyun短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_key: &str,
        sign_name: &str,
        template_id: &str,
        template_map: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_app_config_param_valid(sign_name, template_id, template_map)
            .await?;
        self.setting.load::<TenYunConfig>(None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_key,
                &TenYunTplConfig {
                    template_id: template_id.to_owned(),
                    sign_name: sign_name.to_owned(),
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
pub struct TenyunSenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData> for TenyunSenderTask {
    fn setting_key(&self) -> String {
        TenYunConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<TenYunConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    TenSms::branch_limit()
                } else {
                    e.branch_limit
                }
            })
            .unwrap_or(TenSms::branch_limit())
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
            SettingData::<TenYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to tenyun setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let sub_tpl_config = serde_json::from_str::<TenYunTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to tenyun tpl config fail[{}]:{}",
                    sub_setting.secret_id, e
                ))
            })?;
        debug!(
            "msgid:{}   tpl_config_id:{} sms_app_id:{} tpl:{} sign:{} region:{} var:{}",
            val.sms.id,
            tpl_config.id,
            sub_setting.sms_app_id,
            sub_tpl_config.template_id,
            sub_tpl_config.sign_name,
            sub_setting.region,
            val.sms.tpl_var
        );
        let mobile = sms_data
            .data
            .iter()
            .map(|e| e.mobile.as_str())
            .collect::<Vec<_>>();

        match TenSms::branch_send(
            create_sender_client()?,
            &sub_setting.region,
            &sub_setting.secret_id,
            &sub_setting.secret_key,
            &sub_setting.sms_app_id,
            &sub_tpl_config.sign_name,
            &sub_tpl_config.template_id,
            template_map_to_arr(&val.sms.tpl_var, &sub_tpl_config.template_map),
            &mobile,
        )
        .await
        {
            Ok(resp) => Ok({
                let resp = resp
                    .into_iter()
                    .map(|mut e| {
                        e.send_id = format!("{}{}", e.send_id, e.mobile);
                        e
                    })
                    .collect::<Vec<_>>();

                sms_result_to_task(&sms_data.data, &resp)
            }),
            Err(err) => Err(match err {
                SendError::Next(e) => SenderExecError::Next(e),
                SendError::Finish(e) => SenderExecError::Finish(e),
            }),
        }
    }
}

pub struct TenYunNotify<'t> {
    callback_key: &'t str,
    notify_data: &'t str,
}
impl<'t> TenYunNotify<'t> {
    pub fn new(callback_key: &'t str, notify_data: &'t str) -> TenYunNotify<'t> {
        Self {
            callback_key,
            notify_data,
        }
    }
}

#[async_trait]
impl SmsSendNotifyParse for TenYunNotify<'_> {
    type T = TenYunConfig;
    fn notify_items(
        &self,
        config: &SettingData<TenYunConfig>,
    ) -> Result<Vec<SendNotifyItem>, SendNotifyError> {
        if !config.callback_key.is_empty() && config.callback_key.as_str() != self.callback_key {
            return Err(SendNotifyError::Sign(format!(
                "callback key is match :{}",
                self.callback_key
            )));
        }
        TenSms::send_notify_parse(self.notify_data).map(|resp| {
            resp.into_iter()
                .flat_map(|mut e| match e.mobile.as_ref() {
                    Some(m) => {
                        e.send_id = format!("{}{}", e.send_id, m);
                        Some(e)
                    }
                    None => None,
                })
                .collect::<Vec<_>>()
        })
    }
    fn output(res: &Result<(), String>) -> (u16, String) {
        (200, TenSms::send_notify_output(res))
    }
}

#[derive(Default)]
pub struct TenYunSendStatus {}
#[async_trait]
impl crate::dao::SmsStatusTaskExecutor for TenYunSendStatus {
    fn setting_key(&self) -> String {
        TenYunConfig::key().to_owned()
    }
    async fn exec(
        &self,
        msg: &SenderSmsMessageModel,
        setting: &SettingModel,
    ) -> Result<Vec<SendDetailItem>, SenderExecError> {
        let setting_data =
            SettingData::<TenYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to ten yun setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let naive_date_time = DateTime::from_timestamp(msg.send_time as i64, 0).unwrap_or_default();
        TenSms::send_detail(
            create_sender_client()?,
            &setting_data.region,
            &setting_data.secret_id,
            &setting_data.secret_key,
            &setting_data.sms_app_id,
            &msg.mobile,
            &naive_date_time.date_naive().to_string(),
        )
        .await
        .map(|resp| {
            resp.into_iter()
                .flat_map(|mut e| match e.mobile.as_ref() {
                    Some(m) => {
                        e.send_id = format!("{}{}", e.send_id, m);
                        Some(e)
                    }
                    None => None,
                })
                .collect::<Vec<_>>()
        })
        .map_err(SenderExecError::Next)
    }
}
