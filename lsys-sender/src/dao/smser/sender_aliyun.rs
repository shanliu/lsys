use std::sync::Arc;

use crate::model::SenderSmsMessageModel;
use async_trait::async_trait;
use lsys_core::AppCore;

use sms::aliyun::Aliyun;
use sqlx::{MySql, Pool};
use tracing::debug;

use super::{
    super::task::{TaskError, TaskExecutioner},
    SmsTaskRecord, SmserTaskExecutioner,
};

//aliyun 短信发送

#[derive(Clone)]
pub struct AliyunSender {
    record: SmsTaskRecord,
    app_core: Arc<AppCore>,
}

impl SmserTaskExecutioner<AliyunSender> for AliyunSender {
    fn create(
        app_core: Arc<AppCore>,
        _redis: deadpool_redis::Pool,
        _db: Pool<MySql>,
        record: SmsTaskRecord,
    ) -> Self {
        Self {
            // db,
            record,
            app_core,
        }
    }
}
#[async_trait]
impl TaskExecutioner<u64, SenderSmsMessageModel> for AliyunSender {
    async fn exec(&self, val: SenderSmsMessageModel) -> Result<(), TaskError> {
        let aliconfig = self
            .app_core
            .config
            .get_table("alisms")
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_config]"))?;
        let sms_config = aliconfig
            .get(&val.tpl_id)
            .ok_or_else(|| {
                TaskError::Exec(format!(
                    "not find {} notify config [ali_config]",
                    val.tpl_id
                ))
            })?
            .to_owned()
            .into_table()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_config]"))?;
        let tpl = sms_config
            .get("sms_tpl")
            .ok_or_else(|| {
                TaskError::Exec(format!("not find {} notify tpl [ali_tpls]", val.tpl_id))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_tpls]"))?;
        let key = sms_config
            .get("access_key_id")
            .ok_or_else(|| {
                TaskError::Exec(format!("not find {} notify tpl [ali_key]", val.tpl_id))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_key]"))?;
        let secret = sms_config
            .get("access_key_secret")
            .ok_or_else(|| {
                TaskError::Exec(format!("not find {} notify tpl [ali_secret]", val.tpl_id))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_secret]"))?;
        let sign_name = sms_config
            .get("sign_name")
            .ok_or_else(|| {
                TaskError::Exec(format!(
                    "not find {} notify tpl [ali_sms_sign_name]",
                    val.tpl_id
                ))
            })?
            .to_owned()
            .into_string()
            .map_err(|e| TaskError::Exec(e.to_string() + "[ali_sms_sign_name]"))?;
        let res = match Aliyun::new(&key, &secret)
            .send_sms(&val.mobile, &sign_name, &tpl, &val.tpl_var)
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
            .finish_send("aliyun".to_string(), &val, &res)
            .await
            .map_err(|e| TaskError::Exec(e.to_string()))?;
        res.map_err(TaskError::Exec)
    }
}
