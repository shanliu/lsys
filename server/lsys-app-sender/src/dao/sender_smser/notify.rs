use std::sync::Arc;

use crate::{
    dao::MessageLogs,
    model::{
        SenderLogStatus, SenderSmsBodyModel, SenderSmsMessageModel, SenderSmsMessageStatus,
        SenderType,
    },
};
use lsys_app::dao::AppNotifySender;
use lsys_core::db::SqlQuote;
use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::sql_format;
use lsys_core::IntoFluentMessage;
use lsys_lib_sms::{SendNotifyError, SendNotifyItem, SendNotifyStatus};
use lsys_setting::{
    dao::{SettingData, SettingDecode, SettingKey},
    model::SettingModel,
};
use serde_json::json;
use sqlx::Pool;
use tracing::{info, warn};

pub(crate) async fn add_notify_callback(
    db: &Pool<sqlx::MySql>,
    notify_sender: &AppNotifySender,
    app_id: u64,
    sms_id: u64,
) {
    if app_id == 0 {
        warn!("System SMS Ignore on sms id:{}", sms_id);
        return;
    }

    let sms = match sqlx::query_as::<_, SenderSmsMessageModel>(&sql_format!(
        "select * from {} where id={}",
        SenderSmsMessageModel::table_name(),
        sms_id
    ))
    .fetch_one(db)
    .await
    {
        Ok(m) => m,
        Err(e) => {
            warn!("add notify data fail on select db:{}", e);
            return;
        }
    };
    if let Err(err) = notify_sender
        .send(
            app_id,
            &sms.snid.to_string(),
            &json!({
                "id":sms.id,
                "mobile":sms.mobile,
                "area":sms.area,
                "status":sms.status,
                "receive_time":sms.receive_time,
            })
            .to_string(),
            // 3,
            // AppNotifyTryTimeMode::Exponential,
            // 60,
            // false,
        )
        .await
    {
        warn!(
            "add notify data fail:{}",
            err.to_fluent_message().default_format()
        );
    }
}

//回调接口数据解析trait
pub trait SmsSendNotifyParse {
    type T: SettingDecode;
    fn notify_items(
        &self,
        config: &SettingData<Self::T>,
    ) -> Result<Vec<SendNotifyItem>, SendNotifyError>;
    fn output(res: &Result<(), String>) -> (u16, String);
    fn parse_send_id(&self, items: &[SendNotifyItem]) -> Vec<String> {
        items
            .iter()
            .map(|e| e.send_id.to_owned())
            .collect::<Vec<_>>()
    }
    fn parse_data(
        &self,
        items: &[SendNotifyItem],
        msg: Vec<SenderSmsMessageModel>,
    ) -> Result<Vec<(Option<SenderSmsMessageModel>, SendNotifyItem)>, String> {
        Ok(items
            .iter()
            .map(|e| {
                let tmp = msg
                    .iter()
                    .find(|t| match &e.mobile {
                        Some(_) => t.res_data == e.send_id,
                        None => false,
                    })
                    .map(|t| t.to_owned());
                (tmp, e.to_owned())
            })
            .collect::<Vec<_>>())
    }
}

pub struct SmsSendNotify {
    db: Pool<sqlx::MySql>,
    message_logs: Arc<MessageLogs>,
    notify_sender: Arc<AppNotifySender>,
}

impl SmsSendNotify {
    pub fn new(db: Pool<sqlx::MySql>, notify_sender: Arc<AppNotifySender>) -> Self {
        let message_logs = Arc::new(MessageLogs::new(db.clone(), SenderType::Smser));
        Self {
            db,
            message_logs,
            notify_sender,
        }
    }
    //输出符合指定设配器的结果
    pub fn output<T: SmsSendNotifyParse>(&self, res: &Result<(), String>) -> (u16, String) {
        T::output(res)
    }
    //检查是否是指定配置请求
    pub fn check<T: SmsSendNotifyParse>(&self, config: &SettingModel) -> bool {
        config.setting_key.as_str() == T::T::key()
    }
    //保存短信回调
    pub async fn save<T: SmsSendNotifyParse>(
        &self,
        config: SettingModel,
        data: T,
    ) -> Result<(), String> {
        let sms_config = match SettingData::try_from(config) {
            Ok(c) => c,
            Err(e) => {
                return Err(format!(
                    "parse setting fail:{}",
                    e.to_fluent_message().default_format()
                ));
            }
        };
        let items = data.notify_items(&sms_config).map_err(|e| match e {
            SendNotifyError::Msg(err) => format!("system error:{}", err),
            SendNotifyError::Sign(err) => format!("sign error:{}", err),
            SendNotifyError::Ignore => "".to_string(),
        })?;
        let send_id = data.parse_send_id(&items);
        if send_id.is_empty() {
            return Ok(());
        }

        let msg_data = sqlx::query_as::<_, SenderSmsMessageModel>(&sql_format!(
            "select * from {} where res_data in ({})",
            SenderSmsMessageModel::table_name(),
            send_id
        ))
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;

        let res = data.parse_data(&items, msg_data);
        match res {
            Ok(data) => {
                let findid = data
                    .iter()
                    .flat_map(|e| e.0.as_ref().map(|t| t.sender_body_id))
                    .collect::<Vec<_>>();
                let bodys = if !findid.is_empty() {
                    match sqlx::query_as::<_, SenderSmsBodyModel>(&sql_format!(
                        "select * from {} where id in ({})",
                        SenderSmsBodyModel::table_name(),
                        findid
                    ))
                    .fetch_all(&self.db)
                    .await
                    {
                        Ok(b) => b,
                        Err(e) => {
                            warn!("find sms body fail:{}", e);
                            vec![]
                        }
                    }
                } else {
                    vec![]
                };

                let mut out = Ok(());
                for (mp, n) in data {
                    match mp {
                        Some(m) => {
                            let body = bodys.iter().find(|e| e.id == m.sender_body_id);

                            let mut set_val = vec![];
                            if let Some(t) = n.send_time {
                                if t > 0 {
                                    set_val.push(sql_format!("send_time={}", t))
                                }
                            }
                            let (sql, status, msg) = match n.status {
                                SendNotifyStatus::Completed => {
                                    set_val.push(sql_format!(
                                        "status={}",
                                        SenderSmsMessageStatus::IsReceived as i8
                                    ));
                                    if let Some(t) = n.receive_time {
                                        set_val.push(sql_format!("receive_time={}", t))
                                    }
                                    (
                                        sql_format!(
                                            r#"UPDATE {}
                                            SET {},
                                            WHERE id={};
                                        "#,
                                            SenderSmsMessageModel::table_name(),
                                            SqlExpr(set_val.join(",")),
                                            m.id,
                                        ),
                                        SenderLogStatus::NotifySucc,
                                        n.message,
                                    )
                                }
                                SendNotifyStatus::Failed => {
                                    set_val.push(sql_format!(
                                        "status={}",
                                        SenderSmsMessageStatus::SendFail as i8
                                    ));
                                    (
                                        sql_format!(
                                            r#"UPDATE {}
                                            SET {},
                                            WHERE id={};
                                        "#,
                                            SenderSmsMessageModel::table_name(),
                                            SqlExpr(set_val.join(",")),
                                            m.id,
                                        ),
                                        SenderLogStatus::NotifyFail,
                                        n.message,
                                    )
                                }
                                SendNotifyStatus::Progress => {
                                    info!("sms is sending :{}", m.id);
                                    continue;
                                }
                            };
                            if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
                                warn!("change message status fail[{}]{}", m.id, err);
                                out = Err(err.to_string());
                            }

                            match body {
                                Some(b) => {
                                    //正常解析的回调写日志跟进行回调通知
                                    self.message_logs
                                        .add_exec_log(b.app_id, &[(m.id, status, &msg)], "")
                                        .await;
                                    add_notify_callback(
                                        &self.db,
                                        &self.notify_sender,
                                        b.app_id,
                                        m.id,
                                    )
                                    .await;
                                }
                                None => {
                                    warn!("body is miss. {:?} [{}]", m.id, msg);
                                }
                            }
                        }
                        None => {
                            warn!("not find notify in database. {:?}", n);
                        }
                    }
                }
                out
            }
            Err(e) => Err(e),
        }
    }
}
