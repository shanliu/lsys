use std::sync::Arc;

use crate::{
    dao::{SenderExecError, SenderResult, SenderTaskExecutor, SenderTplConfig, SmsTaskItem},
    model::SenderTplConfigModel,
};
use async_trait::async_trait;
use lsys_core::RequestEnv;
use lsys_setting::{
    dao::{MultipleSetting, SettingData, SettingDecode, SettingEncode, SettingKey, SettingResult},
    model::SettingModel,
};
use serde::{Deserialize, Serialize};

use sms::aliyun::Aliyun;
use tracing::debug;

//aliyun 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct AliYunConfig {
    pub access_id: String,
    pub access_secret: String,
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
        "aliyun-sms-config"
    }
}
impl SettingDecode for AliYunConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        let mut out = data.split(',');
        Ok(AliYunConfig {
            access_id: out.next().unwrap_or_default().to_string(),
            access_secret: out.next().unwrap_or_default().to_string(),
        })
    }
}

impl SettingEncode for AliYunConfig {
    fn encode(&self) -> String {
        format!("{},{}", self.access_id, self.access_secret)
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
        ali_config_ids: &Option<Vec<u64>>,
    ) -> SenderResult<Vec<SettingData<AliYunConfig>>> {
        let data = self
            .setting
            .list_data::<AliYunConfig>(&None, ali_config_ids, &None)
            .await?;
        Ok(data)
    }
    //删除指定的aliyun短信配置
    pub async fn del_config(
        &self,
        id: &u64,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .del::<AliYunConfig>(&None, id, user_id, None, env_data)
            .await?)
    }
    //编辑指定的aliyun短信配置
    pub async fn edit_config(
        &self,
        id: &u64,
        name: &str,
        access_id: &str,
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
                &AliYunConfig {
                    access_id: access_id.to_owned(),
                    access_secret: access_secret.to_owned(),
                },
                user_id,
                None,
                env_data,
            )
            .await?)
    }
    //添加aliyun短信配置
    pub async fn add_config(
        &self,
        name: &str,
        access_id: &str,
        access_secret: &str,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .add(
                &None,
                name,
                &AliYunConfig {
                    access_id: access_id.to_owned(),
                    access_secret: access_secret.to_owned(),
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
        app_id: &u64,
        setting_id: &u64,
        tpl_id: &str,
        aliyun_sms_tpl: &str,
        aliyun_sign_name: &str,
        user_id: &u64,
        add_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.setting.load::<AliYunConfig>(&None, setting_id).await?;
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
impl SenderTaskExecutor<u64, SmsTaskItem> for AliYunSenderTask {
    fn setting_key(&self) -> String {
        AliYunConfig::key().to_owned()
    }
    //执行短信发送
    async fn exec(
        &self,
        val: &SmsTaskItem,
        tpl_config: &SenderTplConfigModel,
        setting: &SettingModel,
    ) -> Result<String, SenderExecError> {
        let ali_config =
            serde_json::from_str::<AliYunTplConfig>(&tpl_config.config_data).map_err(|e| {
                SenderExecError::Next(format!("parse config to aliyun tpl config fail:{}", e))
            })?;
        let ali_setting =
            SettingData::<AliYunConfig>::try_from(setting.to_owned()).map_err(|e| {
                SenderExecError::Next(format!("parse config to aliyun setting fail:{}", e))
            })?;
        debug!(
            "msgid:{}   mobie:{}  tpl_config_id:{} access_id:{} sign_name:{} tpl:{} var:{}",
            val.sms.id,
            val.sms.mobile,
            tpl_config.id,
            ali_setting.access_id,
            ali_config.aliyun_sign_name,
            ali_config.aliyun_sms_tpl,
            val.sms.tpl_var
        );
        match Aliyun::new(&ali_setting.access_id, &ali_setting.access_secret)
            .send_sms(
                &val.sms.mobile,
                &ali_config.aliyun_sign_name,
                &ali_config.aliyun_sms_tpl,
                &val.sms.tpl_var,
            )
            .await
        {
            Ok(resp) => {
                debug!("aliyun sms resp :{:?}", resp);
                if resp.get("Code").map(|e| e == "OK").unwrap_or(false) {
                    Ok(ali_setting.access_id.to_string())
                } else {
                    Err(SenderExecError::Next(format!(
                        "aliyun error:{:?} ",
                        resp.get("Message")
                    )))
                }
            }
            Err(err) => Err(SenderExecError::Next(err.to_string())),
        }
    }
}
