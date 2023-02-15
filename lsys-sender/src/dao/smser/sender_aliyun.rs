use std::{
    collections::{HashMap, HashSet},
    sync::{atomic::AtomicU32, Arc},
};

use crate::{
    dao::task::{TaskAcquisition, TaskRecord, TaskValue},
    model::{
        SenderAliyunConfigModel, SenderAliyunConfigModelRef, SenderAliyunConfigStatus,
        SenderSmsAliyunModel, SenderSmsAliyunModelRef, SenderSmsAliyunStatus,
        SenderSmsMessageModel,
    },
};
use async_trait::async_trait;
use lsys_core::{now_time, AppCore};

use super::{
    super::task::TaskError, SmsTaskAcquisition, SmsTaskItem, SmsTaskRecord, SmserTaskExecutioner,
};
use sms::aliyun::Aliyun;
use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Insert, ModelTableName, Select};
use sqlx_model::{SqlQuote, Update};
use tracing::debug;

//aliyun 短信发送

#[derive(Clone)]
pub struct AliyunSms {
    db: Pool<MySql>,
}

impl AliyunSms {
    pub fn new(db: Pool<sqlx::MySql>) -> Self {
        Self { db }
    }
    //列出有效的aliyun短信配置
    pub async fn list_config(
        &self,
        ali_config_id: &[u64],
    ) -> Result<Vec<SenderAliyunConfigModel>, sqlx::Error> {
        if ali_config_id.is_empty() {
            return Ok(vec![]);
        }
        let sql = sql_format!(
            "id in ({}) and status ={} order by id desc",
            ali_config_id,
            SenderAliyunConfigStatus::Enable
        );
        let ali_res = Select::type_new::<SenderAliyunConfigModel>()
            .fetch_all_by_where::<SenderAliyunConfigModel, _>(
                &sqlx_model::WhereOption::Where(sql),
                &self.db,
            )
            .await?;
        Ok(ali_res)
    }
    //删除指定的aliyun短信配置
    pub async fn del_config(&self, config: &SenderAliyunConfigModel) -> Result<u64, String> {
        let sql = sql_format!(
            "select count(*) as total from {} where aliyun_config_id = {} and status ={}",
            SenderSmsAliyunModel::table_name(),
            config.id,
            SenderSmsAliyunStatus::Enable
        );
        let num = sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await
            .map_err(|e| e.to_string())?;

        if num > 0 {
            return Err("can't be delete,user is used".to_string());
        }

        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderAliyunConfigModelRef,{
            status:SenderAliyunConfigStatus::Delete as i8,
            delete_time:time,
        });
        let res = Update::<sqlx::MySql, SenderAliyunConfigModel, _>::new(change)
            .execute_by_pk(config, &self.db)
            .await;
        match res {
            Err(e) => Err(e.to_string())?,
            Ok(mr) => {
                //清理缓存
                Ok(mr.rows_affected())
            }
        }
    }
    //编辑指定的aliyun短信配置
    pub async fn edit_config(
        &self,
        config: &SenderAliyunConfigModel,
        access_id: &str,
        access_secret: &str,
        user_id: &u64,
    ) -> Result<u64, sqlx::Error> {
        let access_id = access_id.to_owned();
        let access_secret = access_secret.to_owned();
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderAliyunConfigModelRef,{
            access_id:access_id,
            access_secret:access_secret,
            add_time:time,
            user_id:user_id,
        });
        let res = Update::<sqlx::MySql, SenderAliyunConfigModel, _>::new(change)
            .execute_by_pk(config, &self.db)
            .await;
        match res {
            Err(e) => Err(e)?,
            Ok(mr) => {
                //清理缓存
                Ok(mr.rows_affected())
            }
        }
    }
    //添加aliyun短信配置
    pub async fn add_config(
        &self,
        access_id: &str,
        access_secret: &str,
        user_id: &u64,
    ) -> Result<u64, sqlx::Error> {
        let access_id = access_id.to_owned();
        let access_secret = access_secret.to_owned();
        let time = now_time().unwrap_or_default();
        let add = sqlx_model::model_option_set!(SenderAliyunConfigModelRef,{
            access_id:access_id,
            access_secret:access_secret,
            add_time:time,
            user_id:user_id,
            status:SenderAliyunConfigStatus::Enable as i8,
        });
        Insert::<sqlx::MySql, SenderAliyunConfigModel, _>::new(add)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())
    }
    //关联发送跟aliyun短信的配置
    pub async fn add_app_config(
        &self,
        aliyun_config: &SenderAliyunConfigModel,
        sms_tpl: &str,
        aliyun_sms_tpl: &str,
        aliyun_sign_name: &str,
        user_id: &u64,
    ) -> Result<u64, sqlx::Error> {
        let sms_tpl = sms_tpl.to_owned();
        let aliyun_sign_name = aliyun_sign_name.to_owned();
        let aliyun_sms_tpl = aliyun_sms_tpl.to_owned();
        let time = now_time().unwrap_or_default();
        let add = sqlx_model::model_option_set!(SenderSmsAliyunModelRef,{
            sms_tpl:sms_tpl,
            aliyun_sign_name:aliyun_sign_name,
            aliyun_sms_tpl:aliyun_sms_tpl,
            add_time:time,
            user_id:user_id,
            aliyun_config_id:aliyun_config.id,
            status:SenderSmsAliyunStatus::Enable as i8,
        });
        Insert::<sqlx::MySql, SenderSmsAliyunModel, _>::new(add)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())
    }
    //删除发送跟aliyun短信的配置
    pub async fn del_app_config(
        &self,
        sms_aliyun: &SenderSmsAliyunModel,
        user_id: &u64,
    ) -> Result<u64, sqlx::Error> {
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
        app_id: &u64,
    ) -> Result<Vec<(SenderSmsAliyunModel, SenderAliyunConfigModel)>, sqlx::Error> {
        let sql = sql_format!(
            "app_id = {} and status ={} order by id desc",
            app_id,
            SenderSmsAliyunStatus::Enable
        );
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
        let ali_res = self.list_config(&ids).await?;
        if ali_res.is_empty() {
            return Ok(vec![]);
        }
        Ok(res
            .into_iter()
            .filter_map(|r| {
                ali_res
                    .iter()
                    .find(|e| e.id == r.aliyun_config_id)
                    .map(|t| (r, t.to_owned()))
            })
            .collect::<Vec<(SenderSmsAliyunModel, SenderAliyunConfigModel)>>())
    }
}

#[derive(Clone)]
pub struct AliyunSmsRecord {
    records: SmsTaskRecord,
}
impl AliyunSmsRecord {
    pub fn new(try_num: usize, app_core: Arc<AppCore>, db: Pool<sqlx::MySql>) -> Self {
        Self {
            records: SmsTaskRecord::new(db, app_core, try_num),
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
    ) -> Result<TaskRecord<u64, SmsTaskItem<()>>, TaskError> {
        <Self as TaskAcquisition<u64, SmsTaskItem<()>>>::read_record(self, tasking_record, limit)
            .await
    }
}

#[async_trait]
impl SmsTaskAcquisition<()> for AliyunSmsRecord {
    //获取每个发送记录的关联记录数据，阿里云短信没用到，所以返回()
    async fn read_record_attr(
        &self,
        res: Vec<SenderSmsMessageModel>,
    ) -> Result<Vec<SmsTaskItem<()>>, TaskError> {
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
    alisms: AliyunSms,
    i: Arc<AtomicU32>,
}

impl AliyunSenderTask {
    pub fn new(db: Pool<sqlx::MySql>) -> Self {
        Self {
            alisms: AliyunSms::new(db),
            i: Arc::new(AtomicU32::new(0)),
        }
    }
}
#[async_trait]
impl SmserTaskExecutioner<()> for AliyunSenderTask {
    //执行短信发送
    async fn exec(&self, val: SmsTaskItem<()>, record: &SmsTaskRecord) -> Result<(), TaskError> {
        let config = self
            .alisms
            .find_app_config(&val.sms.app_id)
            .await
            .map_err(|e| TaskError::Exec(e.to_string()))?;
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
                .finish("aliyun".to_string(), &val.sms, &res)
                .await
                .map_err(|e| TaskError::Exec(e.to_string()))?;
            return res.map_err(TaskError::Exec);
        }
        Err(TaskError::Exec("not config send".to_string()))
    }
}
