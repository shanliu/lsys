use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::{
        SenderError, SenderExecError, SenderResult, SenderTaskExecutor, SenderTplConfig,
        SmsTaskItem,
    },
    model::SenderTplConfigModel,
};
use async_trait::async_trait;
use lsys_core::{rand_str, RandType, RequestEnv};
use lsys_setting::{
    dao::{
        MultipleSetting, SettingData, SettingDecode, SettingEncode, SettingError, SettingKey,
        SettingResult,
    },
    model::SettingModel,
};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Method, StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use tracing::{debug, warn};

use base64::{
    alphabet,
    engine::{self, general_purpose},
    Engine,
};
use chrono::Utc;
use sha2::{Digest, Sha256};

//华为 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct HwYunConfig {
    pub app_key: String,
    pub app_secret: String,
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
        serde_json::from_str::<Self>(data).map_err(|e| SettingError::System(e.to_string()))
    }
}

impl SettingEncode for HwYunConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct HwYunTplConfig {
    pub url: String,
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
        config_ids: &Option<Vec<u64>>,
    ) -> SenderResult<Vec<SettingData<HwYunConfig>>> {
        let data = self
            .setting
            .list_data::<HwYunConfig>(&None, config_ids, &None)
            .await?;
        Ok(data)
    }
    //删除指定的huawei短信配置
    pub async fn del_config(
        &self,
        id: &u64,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .del::<HwYunConfig>(&None, id, user_id, None, env_data)
            .await?)
    }
    //编辑指定的huawei短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: &u64,
        name: &str,
        app_key: &str,
        app_secret: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .edit(
                &None,
                id,
                name,
                &HwYunConfig {
                    app_key: app_key.to_owned(),
                    app_secret: app_secret.to_owned(),
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //添加huawei短信配置
    pub async fn add_config(
        &self,
        name: &str,
        app_key: &str,
        app_secret: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .add(
                &None,
                name,
                &HwYunConfig {
                    app_key: app_key.to_owned(),
                    app_secret: app_secret.to_owned(),
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //关联发送跟huawei短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        url: &str,
        app_id: &u64,
        setting_id: &u64,
        tpl_id: &str,
        signature: &str,
        sender: &str,
        template_id: &str,
        template_map: &str,
        user_id: &u64,
        add_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err(SenderError::System(format!(
                "your submit url [{}] is wrong",
                url
            )));
        }
        self.setting.load::<HwYunConfig>(&None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_id,
                &HwYunTplConfig {
                    url: url.to_owned(),
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

#[derive(Debug, Clone, Deserialize)]
struct HwYunResponse {
    pub errcode: Option<i64>,
    pub errmsg: Option<String>,
}
impl HwYunResponse {
    pub fn is_success(&self) -> bool {
        self.errcode.unwrap_or(0) == 0
    }
}

const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

//阿里云发送短信后台发送任务实现
#[derive(Clone, Default)]
pub struct HwYunSenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem> for HwYunSenderTask {
    fn setting_key(&self) -> String {
        HwYunConfig::key().to_owned()
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &SmsTaskItem,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> Result<String, SenderExecError> {
        let sub_tpl_config = serde_json::from_str::<HwYunTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!("parse config to huawei tpl config fail:{}", e))
            })?;
        let sub_setting =
            SettingData::<HwYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!("parse config to huawei setting fail:{}", e))
            })?;
        debug!(
            "msgid:{}  mobie:{}  tpl_config_id:{} access_id:{} tpl:{} var:{}",
            val.sms.id,
            val.sms.mobile,
            tpl_config.id,
            sub_setting.app_key,
            sub_tpl_config.template_id,
            val.sms.tpl_var
        );
        let client = reqwest::Client::builder();
        let client = client
            .build()
            .map_err(|e| SenderExecError::Next(format!("hw request client create fail:{}", e)))?;

        let mut headers = HeaderMap::new();

        if let Ok(value) = HeaderValue::from_str("application/x-www-form-urlencoded") {
            headers.insert("Content-Type", value);
        }

        if let Ok(value) =
            HeaderValue::from_str(r#"WSSE realm="SDP",profile="UsernameToken",type="Appkey""#)
        {
            headers.insert("Authorization", value);
        }

        let dt = Utc::now();
        let formatted = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        if let Ok(value) = HeaderValue::from_str(&formatted) {
            headers.insert("X-Sdk-Date", value);
        }

        let rand_s = rand_str(RandType::UpperHex, 32);
        let mut hasher = Sha256::new();
        hasher.update(format!("{}{}{}", &rand_s, &formatted, &sub_setting.app_secret).as_bytes());
        let passdi = CUSTOM_ENGINE.encode(hasher.finalize());

        let sign = format!(
            r#"UsernameToken Username="{}",PasswordDigest="{}",Nonce="{}",Created="{}""#,
            sub_setting.app_key, passdi, rand_s, formatted
        );
        if let Ok(value) = HeaderValue::from_str(&sign) {
            headers.insert("x-wsse", value);
        }

        let mut form_data = vec![];
        form_data.push(format!(
            "from={}",
            serde_urlencoded::to_string(&sub_tpl_config.sender)
                .map_err(|e| { SenderExecError::Finish(format!("url encode from fail:{}", e)) })?
        ));
        form_data.push(format!(
            "to={}",
            serde_urlencoded::to_string(&val.sms.mobile)
                .map_err(|e| { SenderExecError::Finish(format!("url encode to fail:{}", e)) })?
        ));
        form_data.push(format!(
            "templateId={}",
            serde_urlencoded::to_string(&sub_tpl_config.template_id).map_err(|e| {
                SenderExecError::Finish(format!("url encode template_id fail:{}", e))
            })?
        ));
        form_data.push(format!(
            "signature={}",
            serde_urlencoded::to_string(&sub_tpl_config.signature).map_err(|e| {
                SenderExecError::Finish(format!("url encode signature fail:{}", e))
            })?
        ));

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
            if !set_data.is_empty() {
                form_data.push(format!(
                    "templateParas={}",
                    serde_urlencoded::to_string(&set_data).map_err(|e| {
                        SenderExecError::Finish(format!("url encode templateParas fail:{}", e))
                    })?
                ));
            }
        }

        let request = client
            .request(
                Method::POST,
                format!("{}/sms/batchSendSms/v1", sub_tpl_config.url),
            )
            .headers(headers)
            .form(&form_data);
        let result = request
            .send()
            .await
            .map_err(|e| SenderExecError::Next(format!("hw request send fail:{}", e)))?;
        let status = result.status();
        let data = result
            .bytes()
            .await
            .map_err(|e| SenderExecError::Next(format!("hw request read body fail:{}", e)))?;
        let res = unsafe { String::from_utf8_unchecked(data.to_vec()) };
        if status != StatusCode::OK {
            warn!("hw sms response fail: {}", &res);
        } else {
            debug!("hw sms response succ: {}", &res);
        }
        let resp = serde_json::from_str::<HwYunResponse>(&res)
            .map_err(|e| SenderExecError::Next(format!("hw response body parse fail:{}", e)))?;
        if !resp.is_success() {
            return Err(SenderExecError::Next(format!(
                "hw response return fail:{}",
                resp.errmsg.unwrap_or(res)
            )));
        }
        Ok(sub_setting.app_key.to_owned())
    }
}
