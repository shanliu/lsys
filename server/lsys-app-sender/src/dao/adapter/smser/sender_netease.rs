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

use lsys_core::{
    valid_key, IntoFluentMessage, RequestEnv, ValidNumber, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use lsys_lib_sms::{
    template_map_to_arr, NeteaseSms, SendDetailItem, SendError, SendNotifyError, SendNotifyItem,
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
pub struct NetEaseConfig {
    pub access_key: String,
    pub access_secret: String,
    pub branch_limit: u16,
}

impl NetEaseConfig {
    pub fn hide_access_key(&self) -> String {
        let len = self.access_key.chars().count();
        format!(
            "{}**{}",
            self.access_key.chars().take(2).collect::<String>(),
            self.access_key
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

impl SettingKey for NetEaseConfig {
    fn key<'t>() -> &'t str {
        "163-sms-config"
    }
}
impl SettingDecode for NetEaseConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_str::<Self>(data).map_err(SettingError::SerdeJson)
    }
}

impl SettingEncode for NetEaseConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct NetEaseTplConfig {
    pub template_id: String,
    pub template_map: String,
}

//腾讯云发送短信配置
pub struct SenderNetEaseConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderNetEaseConfig {
    pub fn new(setting: Arc<MultipleSetting>, tpl_config: Arc<SenderTplConfig>) -> Self {
        Self {
            setting,
            tpl_config,
        }
    }
    //列出有效的netease短信配置
    pub async fn list_config(
        &self,
        config_ids: Option<&[u64]>,
    ) -> SenderResult<Vec<SettingData<NetEaseConfig>>> {
        let data = self
            .setting
            .list_data::<NetEaseConfig>(None, config_ids, None)
            .await?;
        Ok(data)
    }
    //删除指定的netease短信配置
    pub async fn del_config(
        &self,
        id: u64,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.tpl_config.check_setting_id_used(id).await?;
        Ok(self
            .setting
            .del::<NetEaseConfig>(None, id, user_id, None, env_data)
            .await?)
    }

    #[allow(clippy::too_many_arguments)]
    async fn edit_config_param_valid(
        &self,
        id: u64,
        name: &str,
        access_key: &str,
        access_secret: &str,
        branch_limit: u16,
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
                valid_key!("access_key"),
                &access_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("access_secret"),
                &access_secret,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("branch_limit"),
                &branch_limit,
                &ValidParamCheck::default()
                    .add_rule(ValidNumber::range(1, NeteaseSms::branch_limit())),
            )
            .check()?;
        Ok(())
    }
    //编辑指定的netease短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        access_key: &str,
        access_secret: &str,
        branch_limit: u16,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.edit_config_param_valid(id, name, access_key, access_secret, branch_limit)
            .await?;
        Ok(self
            .setting
            .edit(
                None,
                id,
                &MultipleSettingData {
                    name,
                    data: &NetEaseConfig {
                        access_key: access_key.to_owned(),
                        access_secret: access_secret.to_owned(),
                        branch_limit,
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
        access_key: &str,
        access_secret: &str,
        branch_limit: u16,
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
                valid_key!("access_key"),
                &access_key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
            )
            .add(
                valid_key!("access_secret"),
                &access_secret,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(8, 128)),
            )
            .add(
                valid_key!("branch_limit"),
                &branch_limit,
                &ValidParamCheck::default()
                    .add_rule(ValidNumber::range(1, NeteaseSms::branch_limit())),
            )
            .check()?;
        Ok(())
    }
    //添加netease短信配置
    pub async fn add_config(
        &self,
        name: &str,
        access_key: &str,
        access_secret: &str,
        branch_limit: u16,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_config_param_valid(name, access_key, access_secret, branch_limit)
            .await?;
        Ok(self
            .setting
            .add(
                None,
                &MultipleSettingData {
                    name,
                    data: &NetEaseConfig {
                        access_key: access_key.to_owned(),
                        access_secret: access_secret.to_owned(),
                        branch_limit,
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
            .check()?;
        Ok(())
    }
    //关联发送跟netease短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_key: &str,
        template_id: &str,
        template_map: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_app_config_param_valid(template_id, template_map)
            .await?;
        self.setting.load::<NetEaseConfig>(None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_key,
                &NetEaseTplConfig {
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
pub struct NetEaseSenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData> for NetEaseSenderTask {
    fn setting_key(&self) -> String {
        NetEaseConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<NetEaseConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    NeteaseSms::branch_limit()
                } else {
                    e.branch_limit
                }
            })
            .unwrap_or(NeteaseSms::branch_limit())
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
            SettingData::<NetEaseConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to netease setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let sub_tpl_config = serde_json::from_str::<NetEaseTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to netease tpl config fail[{}]:{}",
                    sub_setting.access_key, e
                ))
            })?;

        debug!(
            "msgid:{}   tpl_config_id:{}  tpl:{} var:{}",
            val.sms.id, tpl_config.id, sub_tpl_config.template_id, val.sms.tpl_var
        );

        let mobile = sms_data
            .data
            .iter()
            .map(|e| e.mobile.as_str())
            .collect::<Vec<_>>();

        match NeteaseSms::branch_send(
            create_sender_client()?,
            &sub_setting.access_key,
            &sub_setting.access_secret,
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

pub struct NetEaseNotify<'t> {
    notify_data: &'t str,
    sign_data: Option<(&'t str, &'t str, &'t str)>,
}
impl<'t> NetEaseNotify<'t> {
    pub fn new(
        notify_data: &'t str,
        // header -> MD5: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx //根据请求中的request body计算出来的MD5值
        // header -> CurTime: 1440570500855    //当前UTC时间戳，从1970年1月1日0点0 分0 秒开始到现在的毫秒数(String)
        // header -> CheckSum: 001511b8435e0b28044ca50a78e8f983026c5e01
        sign_data: Option<(&'t str, &'t str, &'t str)>,
    ) -> NetEaseNotify<'t> {
        Self {
            notify_data,

            sign_data,
        }
    }
}

#[async_trait]
impl SmsSendNotifyParse for NetEaseNotify<'_> {
    type T = NetEaseConfig;
    fn notify_items(
        &self,
        config: &SettingData<NetEaseConfig>,
    ) -> Result<Vec<SendNotifyItem>, SendNotifyError> {
        NeteaseSms::send_notify_parse(
            self.notify_data,
            self.sign_data
                .map(|(header_md5, header_curtime, header_checksum)| {
                    (
                        config.access_secret.as_str(),
                        header_md5,
                        header_curtime,
                        header_checksum,
                    )
                }),
        )
    }
    fn output(res: &Result<(), String>) -> (u16, String) {
        (200, NeteaseSms::send_notify_output(res))
    }
}

#[derive(Default)]
pub struct NetEaseSendStatus {}
#[async_trait]
impl crate::dao::SmsStatusTaskExecutor for NetEaseSendStatus {
    fn setting_key(&self) -> String {
        NetEaseConfig::key().to_owned()
    }
    async fn exec(
        &self,
        msg: &SenderSmsMessageModel,
        setting: &SettingModel,
    ) -> Result<Vec<SendDetailItem>, SenderExecError> {
        let setting_data =
            SettingData::<NetEaseConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to netease setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;

        NeteaseSms::send_detail(
            create_sender_client()?,
            &setting_data.access_key,
            &setting_data.access_secret,
            &msg.res_data,
        )
        .await
        .map_err(SenderExecError::Next)
    }
}
