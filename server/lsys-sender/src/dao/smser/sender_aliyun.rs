use std::sync::{atomic::AtomicU32, Arc};

use crate::{
    dao::{AppConfigReader, SenderError, SenderResult},
    model::{SenderSmsAliyunModel, SenderSmsAliyunModelRef, SenderSmsAliyunStatus},
};
use async_trait::async_trait;
use lsys_core::now_time;
use lsys_setting::dao::{
    MultipleSetting, SettingData, SettingDecode, SettingEncode, SettingKey, SettingResult,
};
use serde::{Deserialize, Serialize};

use super::{SmsTaskItem, SmsTaskRecord, SmserTaskExecutioner};
use sms::aliyun::Aliyun;
use sqlx::{MySql, Pool};
use sqlx_model::Insert;
use sqlx_model::Update;
use tracing::debug;

//aliyun 短信发送

#[derive(Deserialize, Serialize, Clone)]
pub struct AliyunConfig {
    pub access_id: String,
    pub access_secret: String,
}

impl AliyunConfig {
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

#[derive(Serialize)]
pub struct ShowAliyunConfig {
    pub id: u64,
    pub name: String,
    pub access_id: String,
    pub hide_access_id: String,
    pub access_secret: String,
    pub last_user_id: u64,
    pub last_change_time: u64,
}
impl SettingKey for AliyunConfig {
    fn key<'t>() -> &'t str {
        "ali-sms-config"
    }
}
impl SettingDecode for AliyunConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        let mut out = data.split(',');
        Ok(AliyunConfig {
            access_id: out.next().unwrap_or_default().to_string(),
            access_secret: out.next().unwrap_or_default().to_string(),
        })
    }
}

impl SettingEncode for AliyunConfig {
    fn encode(&self) -> String {
        format!("{},{}", self.access_id, self.access_secret)
    }
}

pub struct AliyunSender {
    db: Pool<MySql>,
    setting: Arc<MultipleSetting>,
    app_config_read: AppConfigReader<SenderSmsAliyunModel, AliyunConfig>,
}

impl AliyunSender {
    pub fn new(db: Pool<sqlx::MySql>, setting: Arc<MultipleSetting>) -> Self {
        Self {
            app_config_read: AppConfigReader::new(db.clone(), setting.clone()),
            db,
            setting,
        }
    }
    //列出有效的aliyun短信配置
    pub async fn list_config(
        &self,
        ali_config_ids: &Option<Vec<u64>>,
    ) -> SenderResult<Vec<ShowAliyunConfig>> {
        let data = self
            .setting
            .list_data::<AliyunConfig>(&None, ali_config_ids, &None)
            .await?;
        Ok(data
            .into_iter()
            .map(|e| ShowAliyunConfig {
                id: e.model().id,
                name: e.model().name.to_owned(),
                access_id: e.access_id.to_owned(),
                hide_access_id: e.hide_access_id(),
                access_secret: e.access_secret.to_owned(),
                last_user_id: e.model().last_user_id,
                last_change_time: e.model().last_change_time,
            })
            .collect::<Vec<_>>())
    }
    //删除指定的aliyun短信配置
    pub async fn del_config(&self, id: &u64, user_id: &u64) -> SenderResult<u64> {
        Ok(self.setting.del::<AliyunConfig>(&None, id, user_id).await?)
    }
    //编辑指定的aliyun短信配置
    pub async fn edit_config(
        &self,
        id: &u64,
        name: &str,
        access_id: &str,
        access_secret: &str,
        user_id: &u64,
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .edit(
                &None,
                id,
                name,
                &AliyunConfig {
                    access_id: access_id.to_owned(),
                    access_secret: access_secret.to_owned(),
                },
                user_id,
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
    ) -> SenderResult<u64> {
        Ok(self
            .setting
            .add(
                &None,
                name,
                &AliyunConfig {
                    access_id: access_id.to_owned(),
                    access_secret: access_secret.to_owned(),
                },
                user_id,
            )
            .await?)
    }
    // 配置设置跟app关联
    pub async fn find_app_config_by_id(&self, id: &u64) -> SenderResult<SenderSmsAliyunModel> {
        self.app_config_read.find_by_id(id).await
    }
    //查找指定应用的发送跟aliyun短信的配置
    pub async fn find_app_config(
        &self,
        id: &Option<u64>,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
    ) -> SenderResult<Vec<(SenderSmsAliyunModel, SettingData<AliyunConfig>)>> {
        self.app_config_read
            .list_config(
                id,
                user_id,
                app_id,
                tpl_id,
                &Some(SenderSmsAliyunStatus::Enable as i8),
                None,
                &|e| e.aliyun_config_id,
            )
            .await
    }
    //关联发送跟aliyun短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &String,
        app_id: &u64,
        setting_id: &u64,
        tpl_id: &str,
        aliyun_sms_tpl: &str,
        aliyun_sign_name: &str,
        try_num: &u16,
        user_id: &u64,
        add_user_id: &u64,
    ) -> SenderResult<u64> {
        let aliyun_config = self.setting.load::<AliyunConfig>(&None, setting_id).await?;
        let name = name.to_owned();
        let tpl_id = tpl_id.to_owned();
        let aliyun_sign_name = aliyun_sign_name.to_owned();
        let aliyun_sms_tpl = aliyun_sms_tpl.to_owned();
        let time = now_time().unwrap_or_default();
        let add = sqlx_model::model_option_set!(SenderSmsAliyunModelRef,{
            name:name,
            max_try_num:try_num,
            app_id:app_id,
            tpl_id:tpl_id,
            aliyun_sign_name:aliyun_sign_name,
            aliyun_sms_tpl:aliyun_sms_tpl,
            add_time:time,
            user_id:user_id,
            add_user_id:add_user_id,
            aliyun_config_id:aliyun_config.model().id,
            status:SenderSmsAliyunStatus::Enable as i8,
        });
        Ok(Insert::<sqlx::MySql, SenderSmsAliyunModel, _>::new(add)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())?)
    }
    //删除发送跟aliyun短信的配置
    pub async fn del_app_config(
        &self,
        sms_aliyun: &SenderSmsAliyunModel,
        user_id: &u64,
    ) -> SenderResult<u64> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderSmsAliyunModelRef,{
            status:SenderSmsAliyunStatus::Delete as i8,
            delete_time:time,
            delete_user_id:user_id
        });
        let res = Update::<sqlx::MySql, SenderSmsAliyunModel, _>::new(change)
            .execute_by_pk(sms_aliyun, &self.db)
            .await;
        match res {
            Err(e) => Err(e)?,
            Ok(mr) => {
                //清理缓存
                Ok(mr.rows_affected())
            }
        }
    }
}

#[derive(Clone)]
pub struct AliyunSenderTask {
    alisms: Arc<AliyunSender>,
    i: Arc<AtomicU32>,
}

impl AliyunSenderTask {
    pub fn new(alisms: AliyunSender) -> Self {
        Self {
            alisms: Arc::new(alisms),
            i: Arc::new(AtomicU32::new(0)),
        }
    }
}
#[async_trait]
impl SmserTaskExecutioner<()> for AliyunSenderTask {
    //执行短信发送
    async fn exec(&self, val: SmsTaskItem<()>, record: &SmsTaskRecord) -> SenderResult<()> {
        let config = self
            .alisms
            .find_app_config(
                &None,
                &None,
                &Some(val.sms.app_id),
                &Some(val.sms.tpl_id.clone()),
            )
            .await
            .map_err(|e| SenderError::Exec(e.to_string()))?;
        let len = config.len();
        let now = if self.i.load(std::sync::atomic::Ordering::Relaxed) as usize > len {
            self.i.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        } else {
            self.i.store(0, std::sync::atomic::Ordering::Relaxed);
            0
        } as usize;
        let now = if now > len { len } else { now };
        if let Some(c) = config.get(now) {
            let aliconfig = &c.1;
            let smsconfig = &c.0;

            debug!(
                "msgid:{}  mapid:{} mobie:{} access_id:{} sign_name:{} tpl:{} var:{}",
                val.sms.id,
                val.sms.mobile,
                smsconfig.id,
                aliconfig.access_id,
                smsconfig.aliyun_sign_name,
                smsconfig.aliyun_sms_tpl,
                val.sms.tpl_var
            );

            let res = match Aliyun::new(&aliconfig.access_id, &aliconfig.access_secret)
                .send_sms(
                    &val.sms.mobile,
                    &smsconfig.aliyun_sign_name,
                    &smsconfig.aliyun_sms_tpl,
                    &val.sms.tpl_var,
                )
                .await
            {
                Ok(resp) => {
                    debug!("aliyun sms resp :{:?}", resp);
                    if resp.get("Code").map(|e| e == "OK").unwrap_or(false) {
                        Ok(())
                    } else {
                        Err(format!("aliyun error:{:?} ", resp.get("Message")))
                    }
                }
                Err(err) => Err(err.to_string()),
            };
            record
                .finish(
                    "aliyun".to_string(),
                    aliconfig.access_id.to_string(),
                    &val.sms,
                    &res,
                    smsconfig.max_try_num,
                )
                .await
                .map_err(|e| SenderError::Exec(e.to_string()))?;
            return res.map_err(SenderError::Exec);
        }
        let err = "not find sms config".to_string();
        record
            .finish(
                "aliyun".to_string(),
                "".to_string(),
                &val.sms,
                &Err(err.clone()),
                0,
            )
            .await
            .map_err(|e| SenderError::Exec(e.to_string()))?;
        Err(SenderError::Exec(err))
    }
}
