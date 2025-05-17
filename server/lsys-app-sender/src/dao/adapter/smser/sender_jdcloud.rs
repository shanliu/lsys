use std::sync::Arc;

use crate::{
    dao::{
        adapter::smser::sms_result_to_task, create_sender_client, SenderExecError, SenderResult,
        SenderTaskExecutor, SenderTaskResult, SenderTplConfig, SmsTaskData, SmsTaskItem,
    },
    model::{SenderSmsMessageModel, SenderTplConfigModel},
};
use async_trait::async_trait;

use lsys_core::{
    valid_key, IntoFluentMessage, RequestEnv, ValidNumber, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use lsys_lib_sms::{template_map_to_arr, JdSms, SendDetailItem, SendError};
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
pub struct JDCloudConfig {
    pub region: String,
    pub access_key: String,
    pub access_secret: String,
    pub branch_limit: u16,
}

impl JDCloudConfig {
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

impl SettingKey for JDCloudConfig {
    fn key<'t>() -> &'t str {
        "jd-cloud-sms-config"
    }
}
impl SettingDecode for JDCloudConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        serde_json::from_str::<Self>(data).map_err(SettingError::SerdeJson)
    }
}

impl SettingEncode for JDCloudConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct JDCloudTplConfig {
    pub template_id: String,
    pub sign_id: String,
    pub template_map: String,
}

//腾讯云发送短信配置
pub struct SenderJDCloudConfig {
    setting: Arc<MultipleSetting>,
    tpl_config: Arc<SenderTplConfig>,
}

impl SenderJDCloudConfig {
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
    ) -> SenderResult<Vec<SettingData<JDCloudConfig>>> {
        let data = self
            .setting
            .list_data::<JDCloudConfig>(None, config_ids, None)
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
            .del::<JDCloudConfig>(None, id, user_id, None, env_data)
            .await?)
    }

    #[allow(clippy::too_many_arguments)]
    async fn edit_config_param_valid(
        &self,
        id: u64,
        name: &str,
        region: &str,
        access_key: &str,
        access_secret: &str,
        branch_limit: u16,
    ) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("id"),
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
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, JdSms::branch_limit())),
            )
            .check()?;
        Ok(())
    }

    //编辑指定的jd_cloud短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: u64,
        name: &str,
        region: &str,
        access_key: &str,
        access_secret: &str,
        branch_limit: u16,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.edit_config_param_valid(id, name, region, access_key, access_secret, branch_limit)
            .await?;

        Ok(self
            .setting
            .edit(
                None,
                id,
                &MultipleSettingData {
                    name,
                    data: &JDCloudConfig {
                        region: region.to_owned(),
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
        region: &str,
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
                valid_key!("region"),
                &region,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 128)),
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
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, JdSms::branch_limit())),
            )
            .check()?;
        Ok(())
    }
    //添加jd_cloud短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn add_config(
        &self,
        name: &str,
        region: &str,
        access_key: &str,
        access_secret: &str,
        branch_limit: u16,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_config_param_valid(name, region, access_key, access_secret, branch_limit)
            .await?;
        // if branch_limit > JdSms::branch_limit() {
        //     return Err(SenderError::System(
        //         fluent_message!("sms-config-branch-error",
        //             {"max":  JdSms::branch_limit()}
        //         ),
        //     ));
        // }
        Ok(self
            .setting
            .add(
                None,
                &MultipleSettingData {
                    name,
                    data: &JDCloudConfig {
                        region: region.to_owned(),
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

        sign_id: &str,
        template_id: &str,
        template_map: &str,
    ) -> SenderResult<()> {
        ValidParam::default()
            .add(
                valid_key!("sign_id"),
                &sign_id,
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
    //关联发送跟jd_cloud短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: u64,
        setting_id: u64,
        tpl_id: &str,
        sign_id: &str,
        template_id: &str,
        template_map: &str,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.add_app_config_param_valid(sign_id, template_id, template_map)
            .await?;
        self.setting.load::<JDCloudConfig>(None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_id,
                &JDCloudTplConfig {
                    template_id: template_id.to_owned(),
                    sign_id: sign_id.to_owned(),
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
pub struct JDCloudSenderTask {
    use_jd_cloud: bool,
}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem, SmsTaskData> for JDCloudSenderTask {
    fn setting_key(&self) -> String {
        JDCloudConfig::key().to_owned()
    }
    async fn limit(&self, setting: &SettingModel) -> u16 {
        SettingData::<JDCloudConfig>::try_from(setting.to_owned())
            .map(|e| {
                if e.branch_limit == 0 {
                    JdSms::branch_limit()
                } else {
                    e.branch_limit
                }
            })
            .unwrap_or(JdSms::branch_limit())
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
            SettingData::<JDCloudConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to jd_cloud setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;
        let sub_tpl_config = serde_json::from_str::<JDCloudTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to jd_cloud tpl config fail[{}]:{}",
                    sub_setting.access_key, e
                ))
            })?;
        debug!(
            "msgid:{}  tpl_config_id:{}  tpl:{} sign:{} region:{} var:{}",
            val.sms.id,
            tpl_config.id,
            sub_tpl_config.template_id,
            sub_tpl_config.sign_id,
            sub_setting.region,
            val.sms.tpl_var
        );

        let mobile = sms_data
            .data
            .iter()
            .map(|e| e.mobile.as_str())
            .collect::<Vec<_>>();

        match JdSms::branch_send(
            create_sender_client()?,
            self.use_jd_cloud,
            &sub_setting.region,
            &sub_setting.access_key,
            &sub_setting.access_secret,
            &sub_tpl_config.sign_id,
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

#[derive(Default)]
pub struct JDSendStatus {
    use_jd_cloud: bool,
}
#[async_trait]
impl crate::dao::SmsStatusTaskExecutor for JDSendStatus {
    fn setting_key(&self) -> String {
        JDCloudConfig::key().to_owned()
    }
    async fn exec(
        &self,
        msg: &SenderSmsMessageModel,
        setting: &SettingModel,
    ) -> Result<Vec<SendDetailItem>, SenderExecError> {
        let setting_data =
            SettingData::<JDCloudConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!(
                    "parse config to jd setting fail:{}",
                    e.to_fluent_message().default_format()
                ))
            })?;

        JdSms::send_detail(
            create_sender_client()?,
            self.use_jd_cloud,
            &setting_data.region,
            &setting_data.access_key,
            &setting_data.access_secret,
            &msg.res_data,
            Some(vec![msg.mobile.clone()]),
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
