use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::task::{TaskAcquisition, TaskRecord, TaskValue},
    model::SenderSmsMessageModel,
};
use async_trait::async_trait;
use lsys_core::AppCore;

use sms::aliyun::Aliyun;
use sqlx::{MySql, Pool};
use tracing::debug;

use super::{
    super::task::{TaskError, TaskExecutioner},
    SmsTaskAcquisition, SmsTaskItem, SmsTaskRecord, SmserTaskExecutioner,
};

//aliyun 短信发送

#[derive(Clone)]
pub struct AliyunSmsTaskAcquisition {
    records: SmsTaskRecord,
}
impl AliyunSmsTaskAcquisition {
    pub fn new(try_num: usize, db: Pool<sqlx::MySql>) -> Self {
        Self {
            records: SmsTaskRecord { try_num, db },
        }
    }
}

#[async_trait]
impl TaskAcquisition<u64, SmsTaskItem<()>> for AliyunSmsTaskAcquisition {
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
impl SmsTaskAcquisition<()> for AliyunSmsTaskAcquisition {
    async fn read_record_attr(
        &self,
        res: Vec<SenderSmsMessageModel>,
    ) -> Result<Vec<SmsTaskItem<()>>, TaskError> {
        Ok(res
            .into_iter()
            .map(|e| SmsTaskItem { sms: e, attr: () })
            .collect())
    }
    fn sms_record(&self) -> &SmsTaskRecord {
        &self.records
    }
}

#[derive(Clone)]
pub struct AliyunSender {
    record: SmsTaskRecord,
    app_core: Arc<AppCore>,
}

impl SmserTaskExecutioner<(), AliyunSender> for AliyunSender {
    fn create(
        app_core: Arc<AppCore>,
        _redis: deadpool_redis::Pool,
        _db: Pool<MySql>,
        record: SmsTaskRecord,
    ) -> Self {
        Self { record, app_core }
    }
}

#[async_trait]
impl TaskExecutioner<u64, SmsTaskItem<()>> for AliyunSender {
    async fn exec(&self, val: SmsTaskItem<()>) -> Result<(), TaskError> {
        let aliconfig = self
            .app_core
            .config
            .get_table("alisms")
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_config]"))?;
        let sms_config = aliconfig
            .get(&val.sms.tpl_id)
            .ok_or_else(|| {
                TaskError::Exec(format!(
                    "not find {} notify config [ali_config]",
                    val.sms.tpl_id
                ))
            })?
            .to_owned()
            .into_table()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_config]"))?;
        let tpl = sms_config
            .get("sms_tpl")
            .ok_or_else(|| {
                TaskError::Exec(format!("not find {} notify tpl [ali_tpls]", val.sms.tpl_id))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_tpls]"))?;
        let key = sms_config
            .get("access_key_id")
            .ok_or_else(|| {
                TaskError::Exec(format!("not find {} notify tpl [ali_key]", val.sms.tpl_id))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_key]"))?;
        let secret = sms_config
            .get("access_key_secret")
            .ok_or_else(|| {
                TaskError::Exec(format!(
                    "not find {} notify tpl [ali_secret]",
                    val.sms.tpl_id
                ))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_secret]"))?;
        let sign_name = sms_config
            .get("sign_name")
            .ok_or_else(|| {
                TaskError::Exec(format!(
                    "not find {} notify tpl [ali_sms_sign_name]",
                    val.sms.tpl_id
                ))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_sms_sign_name]"))?;
        let res = match Aliyun::new(&key, &secret)
            .send_sms(&val.sms.mobile, &sign_name, &tpl, &val.sms.tpl_var)
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
        self.record
            .finish("aliyun".to_string(), &val.sms, &res)
            .await
            .map_err(|e| TaskError::Exec(e.to_string()))?;
        res.map_err(TaskError::Exec)
    }
}
