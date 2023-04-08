use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicU32, Arc},
};

use crate::{
    dao::{
        task::{TaskAcquisition, TaskRecord, TaskValue},
        SenderError, SenderResult,
    },
    model::{
        SenderSmsAliyunModel, SenderSmsAliyunModelRef, SenderSmsAliyunStatus, SenderSmsMessageModel,
    },
};
use async_trait::async_trait;
use lsys_core::{now_time, AppCore};
use lsys_setting::dao::{
    MultipleSetting, SettingData, SettingDecode, SettingEncode, SettingKey, SettingResult,
};
use serde::{Deserialize, Serialize};

use super::{SmsTaskAcquisition, SmsTaskItem, SmsTaskRecord, SmserTaskExecutioner};
use sms::aliyun::Aliyun;
use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Insert, Select};
use sqlx_model::{SqlQuote, Update};
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

#[derive(Clone)]
pub struct AliyunSender {
    db: Pool<MySql>,
    setting: Arc<MultipleSetting>,
}

impl AliyunSender {
    pub fn new(db: Pool<sqlx::MySql>, setting: Arc<MultipleSetting>) -> Self {
        Self { db, setting }
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
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_app_config_by_id,
        u64,
        SenderSmsAliyunModel,
        SenderResult<SenderSmsAliyunModel>,
        id,
        "id={id}"
    );
    //关联发送跟aliyun短信的配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_app_config(
        &self,
        name: &String,
        app_id: &u64,
        setting_id: &u64,
        sms_tpl: &str,
        aliyun_sms_tpl: &str,
        aliyun_sign_name: &str,
        try_num: &u16,
        user_id: &u64,
        add_user_id: &u64,
    ) -> SenderResult<u64> {
        let aliyun_config = self.setting.load::<AliyunConfig>(&None, setting_id).await?;
        let name = name.to_owned();
        let sms_tpl = sms_tpl.to_owned();
        let aliyun_sign_name = aliyun_sign_name.to_owned();
        let aliyun_sms_tpl = aliyun_sms_tpl.to_owned();
        let time = now_time().unwrap_or_default();
        let add = sqlx_model::model_option_set!(SenderSmsAliyunModelRef,{
            name:name,
            max_try_num:try_num,
            app_id:app_id,
            sms_tpl:sms_tpl,
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
    //查找指定应用的发送跟aliyun短信的配置
    pub async fn find_app_config(
        &self,
        id: &Option<u64>,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        sms_tpl: &Option<String>,
    ) -> SenderResult<Vec<(SenderSmsAliyunModel, SettingData<AliyunConfig>)>> {
        let mut sqlwhere = vec![sql_format!(" status ={}", SenderSmsAliyunStatus::Enable)];
        if let Some(aid) = id {
            sqlwhere.push(sql_format!("id = {}  ", aid));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("user_id={} ", uid));
        }
        if let Some(tpl) = sms_tpl {
            sqlwhere.push(sql_format!("sms_tpl={} ", tpl));
        }
        let sql = format!("{}  order by id desc", sqlwhere.join(" and "));
        let res = Select::type_new::<SenderSmsAliyunModel>()
            .fetch_all_by_where::<SenderSmsAliyunModel, _>(
                &sqlx_model::WhereOption::Where(sql),
                &self.db,
            )
            .await?;
        if res.is_empty() {
            return Ok(vec![]);
        }
        let ids = res
            .iter()
            .map(|e| e.aliyun_config_id)
            .collect::<HashSet<u64>>()
            .iter()
            .map(|e| e.to_owned())
            .collect::<Vec<u64>>();
        let ali_res = self
            .setting
            .list_data::<AliyunConfig>(&None, &Some(ids), &None)
            .await?;
        if ali_res.is_empty() {
            return Ok(vec![]);
        }
        let out = res
            .into_iter()
            .filter_map(|r| {
                ali_res
                    .iter()
                    .find(|e| e.model().id == r.aliyun_config_id)
                    .map(|t| (r, t.to_owned()))
            })
            .collect::<Vec<(_, _)>>();
        Ok(out)
    }
}

#[derive(Clone)]
pub struct AliyunSmsRecord {
    records: SmsTaskRecord,
}
impl AliyunSmsRecord {
    pub fn new(app_core: Arc<AppCore>, db: Pool<sqlx::MySql>) -> Self {
        Self {
            records: SmsTaskRecord::new(db, app_core),
        }
    }
}

#[async_trait]
impl TaskAcquisition<u64, SmsTaskItem<()>> for AliyunSmsRecord {
    //复用父结构体方法实现
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskValue>,
        limit: usize,
    ) -> SenderResult<TaskRecord<u64, SmsTaskItem<()>>> {
        SmsTaskAcquisition::read_record(self, tasking_record, limit).await
    }
}

#[async_trait]
impl SmsTaskAcquisition<()> for AliyunSmsRecord {
    //获取每个发送记录的关联记录数据，阿里云短信没用到，所以返回()
    async fn read_record_attr(
        &self,
        res: Vec<SenderSmsMessageModel>,
    ) -> SenderResult<Vec<SmsTaskItem<()>>> {
        Ok(res
            .into_iter()
            .map(|e| SmsTaskItem { sms: e, attr: () })
            .collect())
    }
    //短信管理引用
    fn sms_record(&self) -> &SmsTaskRecord {
        &self.records
    }
}
#[derive(Clone)]
pub struct AliyunSenderTask {
    alisms: AliyunSender,
    i: Arc<AtomicU32>,
}

impl AliyunSenderTask {
    pub fn new(alisms: AliyunSender) -> Self {
        Self {
            alisms,
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
        for (i, c) in config.iter().enumerate() {
            if i != now {
                continue;
            }

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
                .finish("aliyun".to_string(), &val.sms, &res, smsconfig.max_try_num)
                .await
                .map_err(|e| SenderError::Exec(e.to_string()))?;
            return res.map_err(SenderError::Exec);
        }
        let err = "not find sms config".to_string();
        record
            .finish("aliyun".to_string(), &val.sms, &Err(err.clone()), 0)
            .await
            .map_err(|e| SenderError::Exec(e.to_string()))?;
        Err(SenderError::Exec(err))
    }
}
