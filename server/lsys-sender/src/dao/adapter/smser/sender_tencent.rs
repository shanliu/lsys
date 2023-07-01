use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::{SenderExecError, SenderResult, SenderTaskExecutor, SenderTplConfig, SmsTaskItem},
    model::SenderTplConfigModel,
};
use async_trait::async_trait;
use hmac::{Hmac, Mac};
use lsys_core::{now_time, RequestEnv};
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

//腾讯云 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct TenYunConfig {
    pub secret_id: String,
    pub secret_key: String,
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
        serde_json::from_str::<Self>(data).map_err(|e| SettingError::System(e.to_string()))
    }
}

impl SettingEncode for TenYunConfig {
    fn encode(&self) -> String {
        json!(self).to_string()
    }
}

#[derive(Deserialize, Serialize, Clone)]
pub struct TenYunTplConfig {
    pub region: String,
    pub sms_app_id: String,
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
        config_ids: &Option<Vec<u64>>,
    ) -> SenderResult<Vec<SettingData<TenYunConfig>>> {
        let data = self
            .setting
            .list_data::<TenYunConfig>(&None, config_ids, &None)
            .await?;
        Ok(data)
    }
    //删除指定的tenyun短信配置
    pub async fn del_config(
        &self,
        id: &u64,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .del::<TenYunConfig>(&None, id, user_id, None, env_data)
            .await?)
    }
    //编辑指定的tenyun短信配置

    #[allow(clippy::too_many_arguments)]
    pub async fn edit_config(
        &self,
        id: &u64,
        name: &str,
        secret_id: &str,
        secret_key: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .edit(
                &None,
                id,
                name,
                &TenYunConfig {
                    secret_id: secret_id.to_owned(),
                    secret_key: secret_key.to_owned(),
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //添加tenyun短信配置
    pub async fn add_config(
        &self,
        name: &str,
        secret_id: &str,
        secret_key: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .add(
                &None,
                name,
                &TenYunConfig {
                    secret_id: secret_id.to_owned(),
                    secret_key: secret_key.to_owned(),
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //关联发送跟tenyun短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &str,
        app_id: &u64,
        setting_id: &u64,
        tpl_id: &str,
        region: &str,
        sms_app_id: &str,
        sign_name: &str,
        template_id: &str,
        template_map: &str,
        user_id: &u64,
        add_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.setting.load::<TenYunConfig>(&None, setting_id).await?;
        self.tpl_config
            .add_config(
                name,
                app_id,
                setting_id,
                tpl_id,
                &TenYunTplConfig {
                    region: region.to_owned(),
                    sms_app_id: sms_app_id.to_owned(),
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

// {
//     "Response": {
//         "SendStatusSet": [
//             {
//                 "SerialNo": "5000:1045710669157053657849499619",
//                 "PhoneNumber": "+8618511122233",
//                 "Fee": 1,
//                 "SessionContext": "test",
//                 "Code": "Ok",
//                 "Message": "send success",
//                 "IsoCode": "CN"
//             },
//             {
//                 "SerialNo": "5000:1045710669157053657849499718",
//                 "PhoneNumber": "+8618511122266",
//                 "Fee": 1,
//                 "SessionContext": "test",
//                 "Code": "Ok",
//                 "Message": "send success",
//                 "IsoCode": "CN"
//             }
//         ],
//         "RequestId": "a0aabda6-cf91-4f3e-a81f-9198114a2279"
//     }
// }
#[derive(Debug, Clone, Deserialize)]
struct TenyunResponseItem {
    #[serde(rename = "Code")]
    pub code: String,
    #[serde(rename = "Message")]
    pub message: String,
}
#[derive(Debug, Clone, Deserialize)]
struct TenyunResponse {
    #[serde(rename = "Response")]
    pub response: Vec<TenyunResponseItem>,
}
impl TenyunResponse {
    pub fn is_success(&self) -> bool {
        if let Some(dat) = self.response.get(0) {
            return dat.code == "Ok";
        }
        false
    }
    pub fn message(&self) -> &str {
        if let Some(dat) = self.response.get(0) {
            return &dat.message;
        }
        "not find any send data"
    }
}
use chrono::NaiveDateTime;
use sha2::{Digest, Sha256};
//腾讯云发送短信后台发送任务实现
#[derive(Clone, Default)]
pub struct TenyunSenderTask {}

#[async_trait]
impl SenderTaskExecutor<u64, SmsTaskItem> for TenyunSenderTask {
    fn setting_key(&self) -> String {
        TenYunConfig::key().to_owned()
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &SmsTaskItem,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> Result<String, SenderExecError> {
        let sub_tpl_config = serde_json::from_str::<TenYunTplConfig>(&tpl_config.config_data)
            .map_err(|e| {
                SenderExecError::Next(format!("parse config to tenyun tpl config fail:{}", e))
            })?;
        let sub_setting =
            SettingData::<TenYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!("parse config to tenyun setting fail:{}", e))
            })?;
        debug!(
            "msgid:{}  mobie:{}  tpl_config_id:{} sms_app_id:{} tpl:{} sign:{} region:{} var:{}",
            val.sms.id,
            val.sms.mobile,
            tpl_config.id,
            sub_tpl_config.sms_app_id,
            sub_tpl_config.template_id,
            sub_tpl_config.sign_name,
            sub_tpl_config.region,
            val.sms.tpl_var
        );
        let client = reqwest::Client::builder();
        let client = client.build().map_err(|e| {
            SenderExecError::Next(format!("Tenyun request client create fail:{}", e))
        })?;
        let now_time = now_time().unwrap_or_default();
        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str("sms.tencentcloudapi.com") {
            headers.insert("Host", value);
        }
        if let Ok(value) = HeaderValue::from_str("application/json; charset=utf-8") {
            headers.insert("Content-Type", value);
        }
        if let Ok(value) = HeaderValue::from_str(&now_time.to_string()) {
            headers.insert("X-TC-Timestamp", value);
        }
        if let Ok(value) = HeaderValue::from_str(&sub_tpl_config.region) {
            headers.insert("X-TC-Region", value);
        }
        if let Ok(value) = HeaderValue::from_str("2017-03-12") {
            headers.insert("X-TC-Version", value);
        }
        if let Ok(value) = HeaderValue::from_str("SendSms") {
            headers.insert("X-TC-Action", value);
        }
        if let Ok(value) = HeaderValue::from_str("zh-CN") {
            headers.insert("X-TC-Language", value);
        }

        let datetime = NaiveDateTime::from_timestamp_opt(now_time as i64, 0).unwrap_or_default();

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
            "PhoneNumberSet": [
                val.sms.mobile,
            ],
            "SmsSdkAppId": sub_tpl_config.sms_app_id,
            "SignName": sub_tpl_config.sign_name,
            "TemplateId": sub_tpl_config.template_id,
            "TemplateParamSet": var_data,
        })
        .to_string();

        let mut hasher = Sha256::new();
        hasher.update(&reqjson);
        let result = hasher.finalize();

        let sign = format!(
            r#"POST
/

content-type:application/json; charset=utf-8
host:sms.tencentcloudapi.com
x-tc-action:sendsms

content-type;host;x-tc-action
{}"#,
            format!("{:x}", result).to_lowercase()
        );

        let mut hasher1 = Sha256::new();
        hasher1.update(&sign);
        let result = hasher1.finalize();

        let string_to_sign = format!(
            r#"TC3-HMAC-SHA256
{}
{}/sms/tc3_request
{}"#,
            now_time,
            datetime.format("%Y-%m-%d"),
            format!("{:x}", result).to_lowercase()
        );

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(
            datetime.format("%Y-%m-%d").to_string().as_bytes(),
        )
        .map_err(|e| SenderExecError::Finish(format!("use data key on sha256 fail:{}", e)))?;
        mac.update(format!("TC3{}", sub_setting.secret_key).as_bytes());
        let result = mac.finalize();

        let mut mac1 = Hmac::<sha2::Sha256>::new_from_slice(b"sms")
            .map_err(|e| SenderExecError::Finish(format!("use sms on sha256 fail:{}", e)))?;
        mac1.update(&result.into_bytes());
        let result = mac1.finalize();

        let mut mac2 = Hmac::<sha2::Sha256>::new_from_slice(b"tc3_request").map_err(|e| {
            SenderExecError::Finish(format!("use tc3_request on sha256 fail:{}", e))
        })?;
        mac2.update(&result.into_bytes());
        let secret_signing = mac2.finalize();

        let mut tmp3 =
            Hmac::<sha2::Sha256>::new_from_slice(string_to_sign.as_bytes()).map_err(|e| {
                SenderExecError::Finish(format!("use tc3_request on sha256 fail:{}", e))
            })?;
        tmp3.update(&secret_signing.into_bytes());
        let signature = tmp3.finalize();

        let authdata =format!(
            "TC3-HMAC-SHA256 Credential={}/{}/sms/tc3_request, SignedHeaders=content-type;host;x-tc-action, Signature={}",
            sub_setting.secret_id,
            datetime.format("%Y-%m-%d"),
            hex::encode(signature.into_bytes())
        );
        if let Ok(value) = HeaderValue::from_str(&authdata) {
            headers.insert("Authorization", value);
        }

        let request = client
            .request(Method::POST, "https://sms.tencentcloudapi.com/")
            .headers(headers)
            .form(&reqjson);
        let result = request
            .send()
            .await
            .map_err(|e| SenderExecError::Next(format!("Tenyun request send fail:{}", e)))?;
        let status = result.status();
        let data = result
            .bytes()
            .await
            .map_err(|e| SenderExecError::Next(format!("Tenyun request read body fail:{}", e)))?;
        let res = unsafe { String::from_utf8_unchecked(data.to_vec()) };
        if status != StatusCode::OK {
            warn!("Tenyun sms response fail: {}", &res);
        } else {
            debug!("Tenyun sms response succ: {}", &res);
        }
        let resp = serde_json::from_str::<TenyunResponse>(&res)
            .map_err(|e| SenderExecError::Next(format!("Tenyun response body parse fail:{}", e)))?;
        if !resp.is_success() {
            return Err(SenderExecError::Next(format!(
                "Tenyun response return fail:{}",
                resp.message()
            )));
        }
        Ok(format!(
            "{}-{}",
            sub_setting.secret_id, sub_tpl_config.sms_app_id
        ))
    }
}
