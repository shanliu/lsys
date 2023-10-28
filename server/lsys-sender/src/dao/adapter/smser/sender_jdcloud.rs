use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::{SenderExecError, SenderResult, SenderTaskExecutor, SenderTplConfig, SmsTaskItem},
    model::SenderTplConfigModel,
};
use async_trait::async_trait;
use http::Request;
use jdcloud_signer::{Client, Credential, Signer};
use lsys_core::RequestEnv;
use lsys_setting::{
    dao::{
        MultipleSetting, SettingData, SettingDecode, SettingEncode, SettingError, SettingKey,
        SettingResult,
    },
    model::SettingModel,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::debug;
//腾讯云 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct JDCloudConfig {
    pub access_key: String,
    pub access_secret: String,
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
        serde_json::from_str::<Self>(data).map_err(|e| SettingError::System(e.to_string()))
    }
}

impl SettingEncode for JDCloudConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct JDCloudTplConfig {
    pub sms_app_id: String,
    pub region: String,
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
        config_ids: &Option<Vec<u64>>,
    ) -> SenderResult<Vec<SettingData<JDCloudConfig>>> {
        let data = self
            .setting
            .list_data::<JDCloudConfig>(&None, config_ids, &None)
            .await?;
        Ok(data)
    }
    //删除指定的jd_cloud短信配置
    pub async fn del_config(
        &self,
        id: &u64,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .del::<JDCloudConfig>(&None, id, user_id, None, env_data)
            .await?)
    }
    //编辑指定的jd_cloud短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: &u64,
        name: &str,
        access_key: &str,
        access_secret: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .edit(
                &None,
                id,
                name,
                &JDCloudConfig {
                    access_key: access_key.to_owned(),
                    access_secret: access_secret.to_owned(),
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //添加jd_cloud短信配置
    pub async fn add_config(
        &self,
        name: &str,
        access_key: &str,
        access_secret: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .add(
                &None,
                name,
                &JDCloudConfig {
                    access_key: access_key.to_owned(),
                    access_secret: access_secret.to_owned(),
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
        app_id: &u64,
        setting_id: &u64,
        tpl_id: &str,
        region: &str,
        sms_app_id: &str,
        sign_id: &str,
        template_id: &str,
        template_map: &str,
        user_id: &u64,
        add_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.setting
            .load::<JDCloudConfig>(&None, setting_id)
            .await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_id,
                &JDCloudTplConfig {
                    region: region.to_owned(),
                    sms_app_id: sms_app_id.to_owned(),
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
pub struct JDCloudSenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem> for JDCloudSenderTask {
    fn setting_key(&self) -> String {
        JDCloudConfig::key().to_owned()
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &SmsTaskItem,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> Result<String, SenderExecError> {
        let sub_tpl_config = serde_json::from_str::<JDCloudTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!("parse config to jd_cloud tpl config fail:{}", e))
            })?;
        let sub_setting =
            SettingData::<JDCloudConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!("parse config to jd_cloud setting fail:{}", e))
            })?;

        debug!(
            "msgid:{}  mobie:{}  tpl_config_id:{} sms_app_id:{} tpl:{} sign:{} region:{} var:{}",
            val.sms.id,
            val.sms.mobile,
            tpl_config.id,
            sub_tpl_config.sms_app_id,
            sub_tpl_config.template_id,
            sub_tpl_config.sign_id,
            sub_tpl_config.region,
            val.sms.tpl_var
        );

        let var_data =
            if let Ok(tmp) = serde_json::from_str::<HashMap<String, String>>(&val.sms.tpl_var) {
                let map_data = sub_tpl_config.template_map.split(',');
                let mut set_data = vec![];
                if !tmp.is_empty() {
                    for sp in map_data {
                        if let Some(tv) = tmp.get(sp) {
                            set_data.push(tv.to_owned())
                        }
                    }
                }
                set_data
            } else {
                vec![]
            };

        let reqjson = json!({
            "templateId" :sub_tpl_config.template_id,
            "signId" : sub_tpl_config.sign_id,
            "phoneList": [
                val.sms.mobile,
            ],
            "params" :var_data,
        })
        .to_string();

        let url = format!(
            "https://sms.jdcloud-api.com/v1/regions/{}/batchSend",
            sub_tpl_config.region
        );

        let credential = Credential::new(&sub_setting.access_key, &sub_setting.access_secret);
        let signer = Signer::new(credential, "sms".to_string(), sub_tpl_config.region);

        let mut req = Request::builder();
        let mut req = req
            .method("POST")
            .uri(url)
            .body(reqjson)
            .map_err(|e| SenderExecError::Finish(format!("creata req fail:{}", e)))?;
        signer
            .sign_request(&mut req)
            .map_err(|e| SenderExecError::Finish(format!(" sign fail:{}", e)))?;
        let client = Client::new();
        let mut res = client
            .execute(req)
            .map_err(|e| SenderExecError::Finish(format!(" req fail:{}", e)))?;
        if !res.status().is_success() {
            return Err(SenderExecError::Next(format!(
                "JDCloud response return fail:{:?}",
                res.text()
            )));
        }
        Ok(format!(
            "{}-{}",
            sub_setting.access_key, sub_tpl_config.sms_app_id
        ))
    }
}
