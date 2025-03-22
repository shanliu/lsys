use std::sync::Arc;

use crate::{
    dao::{
        adapter::smser::sms_result_to_task, create_sender_client, SenderError, SenderExecError,
        SenderResult, SenderTaskExecutor, SenderTaskResult, SenderTplConfig, SmsSendNotifyParse,
        SmsTaskData, SmsTaskItem,
    },
    model::{SenderSmsMessageModel, SenderTplConfigModel},
};
use async_trait::async_trait;
use chrono::DateTime;
use lsys_core::{fluent_message, IntoFluentMessage, RequestEnv};
use lsys_lib_sms::{AliSms, SendDetailItem, SendError, SendNotifyError};
use lsys_setting::{
    dao::{
        MultipleSetting, MultipleSettingData, SettingData, SettingDecode, SettingEncode,
        SettingError, SettingKey, SettingResult,
    },
    model::SettingModel,
};
use serde::{Deserialize, Serialize};

use lsys_lib_sms::SendNotifyItem;
use serde_json::json;
use tracing::debug;
//aliyun 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct AliYunConfig {
    pub access_id: String,
    pub access_secret: String,
    pub region: String,
    pub branch_limit: u16,
    pub callback_key: String,
}

impl AliYunConfig {
    pub fn hide_access_id(&self) -> String {
        let len = self.access_id.chars().count();
        format!(
            "{}**{}",
            self.access_id.chars().take(2).collect::<String>(),
            self.access_id
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
    pub fn hide_access_secret(&self) -> String {
        let len = self.access_secret.chars().count();
        format!(
            "{}**{}",
            self.access_secret.chars().take(2).collect::<String>(),
            self.access_secret
                .chars()
                .skip(if len - 2 > 0 { len - 2 } else { len })
                .take(2)
                .collect::<String>()
        )
    }
}

impl SettingKey for AliYunConfig {
    fn key<'t>() -> &'t str {
        "ali-sms-config"
    }
}
impl SettingDecode for AliYunConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_str::<Self>(data).map_err(SettingError::SerdeJson)
    }
}

impl SettingEncode for AliYunConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AliYunTplConfig {
    pub aliyun_sms_tpl: String,
    pub aliyun_sign_name: String,
}

//阿里云发送短信配置
pub struct SenderAliYunConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderAliYunConfig {
    pub fn new(setting: Arc<MultipleSetting>, tpl_config: Arc<SenderTplConfig>) -> Self {
        Self {
            setting,
            tpl_config,
        }
    }
    //列出有效的aliyun短信配置
    pub async fn list_config(
        &self,
        ali_config_ids: Option<&[u64]>,
    ) -> SenderResult<Vec<SettingData<AliYunConfig>>> {
        let data = self
            .setting
            .list_data::<AliYunConfig>(None, ali_config_ids, None)
            .await?;
        Ok(data)
    }
    //删除指定的aliyun短信配置
    pub async fn del_config(
        &self,
        id: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .del::<AliYunConfig>(None, id, user_id, None, env_data)
            .await?)
    }
    //编辑指定的aliyun短信配置
    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        access_id: &str,
        access_secret: &str,
        region: &str,
        callback_key: &str,
        branch_limit: u16,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        if branch_limit > AliSms::branch_limit() {
            return Err(SenderError::System(
                fluent_message!("sms-config-branch-error",
                    {"max":AliSms::branch_limit()}
                ),
            )); //"limit max:{}",
        }
        Ok(self
            .setting
            .edit(
                None,
                id,
                &MultipleSettingData {
                    name,
                    data: &AliYunConfig {
                        access_id: access_id.to_owned(),
                        access_secret: access_secret.to_owned(),
                        region: region.to_owned(),
                        branch_limit,
                        callback_key: callback_key.to_string(),
                    },
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //添加aliyun短信配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_config(
        &self,
        name: &str,
        access_id: &str,
        access_secret: &str,
        region: &str,
        callback_key: &str,
        branch_limit: u16,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        if branch_limit > AliSms::branch_limit() {
            return Err(SenderError::System(
                fluent_message!("sms-config-branch-error",
                    {"max":AliSms::branch_limit()}
                ),
            ));
        }
        Ok(self
            .setting
            .add(
                None,
                &MultipleSettingData {
                    name,
                    data: &AliYunConfig {
                        access_id: access_id.to_owned(),
                        access_secret: access_secret.to_owned(),
                        region: region.to_owned(),
                        branch_limit,
                        callback_key: callback_key.to_string(),
                    },
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //关联发送跟aliyun短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_id: &str,
        aliyun_sms_tpl: &str,
        aliyun_sign_name: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.setting.load::<AliYunConfig>(None, setting_id).await?;
        let aliyun_sms_tpl = aliyun_sms_tpl.to_owned();
        let aliyun_sign_name = aliyun_sign_name.to_owned();
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_id,
                &AliYunTplConfig {
                    aliyun_sms_tpl,
                    aliyun_sign_name,
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
pub struct AliYunSenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData> for AliYunSenderTask {
    fn setting_key(&self) -> String {
        AliYunConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<AliYunConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    AliSms::branch_limit()
                } else {
                    e.branch_limit
                }
            })
            .unwrap_or(AliSms::branch_limit())
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &SmsTaskItem,
        sms_data: &SmsTaskData,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> SenderTaskResult {
        let ali_setting =
            SettingData::<AliYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to aliyun setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let ali_config =
            serde_json::from_str::<AliYunTplConfig>(&tpl_config.config_data).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to aliyun tpl config fail[{}]:{}",
                    ali_setting.access_id, e
                ))
            })?;
        debug!(
            "msgid:{}  tpl_config_id:{} access_id:{} sign_name:{} tpl:{} var:{}",
            val.sms.id,
            tpl_config.id,
            ali_setting.access_id,
            ali_config.aliyun_sign_name,
            ali_config.aliyun_sms_tpl,
            val.sms.tpl_var
        );
        let mobile = sms_data
            .data
            .iter()
            .map(|e| e.mobile.as_str())
            .collect::<Vec<_>>();

        match AliSms::branch_send(
            create_sender_client()?,
            &ali_setting.region,
            &ali_setting.access_id,
            &ali_setting.access_secret,
            &ali_config.aliyun_sign_name,
            &ali_config.aliyun_sms_tpl,
            &val.sms.tpl_var,
            &mobile,
            "",
            &val.sms.user_ip,
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

pub struct AliYunNotify<'t> {
    callback_key: &'t str,
    notify_data: &'t str,
}

impl<'t> AliYunNotify<'t> {
    pub fn new(callback_key: &'t str, notify_data: &'t str) -> AliYunNotify<'t> {
        Self {
            callback_key,
            notify_data,
        }
    }
}

impl SmsSendNotifyParse for AliYunNotify<'_> {
    type T = AliYunConfig;
    fn notify_items(
        &self,
        config: &SettingData<AliYunConfig>,
    ) -> Result<Vec<SendNotifyItem>, SendNotifyError> {
        if !config.callback_key.is_empty() && config.callback_key.as_str() != self.callback_key {
            return Err(SendNotifyError::Sign(format!(
                "callback key is match :{}",
                self.callback_key
            )));
        }
        AliSms::send_notify_parse(self.notify_data)
    }
    fn output(res: &Result<(), String>) -> (u16, String) {
        (200, AliSms::send_notify_output(res))
    }
    fn parse_send_id(&self, items: &[SendNotifyItem]) -> Vec<String> {
        items
            .iter()
            .map(|e| format!("{}{}", e.send_id, e.mobile.as_deref().unwrap_or_default()))
            .collect::<Vec<_>>()
    }
    fn parse_data(
        &self,
        items: &[SendNotifyItem],
        msg: Vec<SenderSmsMessageModel>,
    ) -> Result<Vec<(Option<SenderSmsMessageModel>, SendNotifyItem)>, String> {
        Ok(items
            .iter()
            .map(|e| {
                let tmp = msg
                    .iter()
                    .find(|t| match &e.mobile {
                        Some(m) => t.res_data == format!("{}{}", e.send_id, m),
                        None => false,
                    })
                    .map(|t| t.to_owned());
                (tmp, e.to_owned())
            })
            .collect::<Vec<_>>())
    }
}

#[derive(Default)]
pub struct AliYunSendStatus {}
#[async_trait]
impl crate::dao::SmsStatusTaskExecutor for AliYunSendStatus {
    fn setting_key(&self) -> String {
        AliYunConfig::key().to_owned()
    }
    async fn exec(
        &self,
        msg: &SenderSmsMessageModel,
        setting: &SettingModel,
    ) -> Result<Vec<SendDetailItem>, SenderExecError> {
        let ali_setting =
            SettingData::<AliYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to aliyun setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let naive_date_time = DateTime::from_timestamp(msg.send_time as i64, 0).unwrap_or_default();

        AliSms::send_detail(
            create_sender_client()?,
            &ali_setting.access_id,
            &ali_setting.access_secret,
            &msg.res_data,
            &msg.mobile,
            &naive_date_time.date_naive().to_string(),
        )
        .await
        .map(|e| {
            e.into_iter()
                .flat_map(|mut m| match m.mobile.as_ref() {
                    Some(mm) => {
                        m.send_id = format!("{}{}", m.send_id, mm);
                        Some(m)
                    }
                    None => None,
                })
                .collect()
        })
        .map_err(SenderExecError::Next)
    }
}
