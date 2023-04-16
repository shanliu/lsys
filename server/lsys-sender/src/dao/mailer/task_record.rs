use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::{
        MessageCancel, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
        TaskAcquisition, TaskRecord,
    },
    model::{
        SenderConfigModel, SenderKeyCancelModel, SenderLogModel, SenderMailConfigData,
        SenderMailConfigLimit, SenderMailConfigType, SenderMailMessageModel,
        SenderMailMessageModelRef, SenderMailMessageStatus, SenderType,
    },
};
use async_trait::async_trait;
use lsys_core::{now_time, AppCore, FluentMessage, PageParam};

use serde_json::Value;
use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Insert, ModelTableName, SqlExpr, Update};

use super::{MailTaskAcquisition, MailTaskItem, TaskData};
use sqlx_model::SqlQuote;

//短信任务记录

pub struct MailTaskRecord {
    pub(crate) db: Pool<sqlx::MySql>,
    pub(crate) config: SenderConfig,
    pub(crate) cancel: MessageCancel,
    pub(crate) logs: MessageLogs,
    pub(crate) record: MessageReader<SenderMailMessageModel>,
}

impl MailTaskRecord {
    pub fn new(db: Pool<sqlx::MySql>, app_core: Arc<AppCore>, fluent: Arc<FluentMessage>) -> Self {
        Self {
            config: SenderConfig::new(db.clone(), SenderType::Mailer),
            cancel: MessageCancel::new(db.clone(), SenderType::Mailer),
            logs: MessageLogs::new(db.clone(), SenderType::Mailer),
            record: MessageReader::new(db.clone(), app_core, fluent),
            db,
        }
    }
    //读取邮件任务数据
    pub async fn read(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> SenderResult<(Vec<SenderMailMessageModel>, bool)> {
        self.record
            .read(tasking_record, SenderMailMessageStatus::Init as i8, limit)
            .await
    }
    pub async fn find_message_by_id(&self, id: &u64) -> SenderResult<SenderMailMessageModel> {
        self.record.find_message_by_id(id).await
    }
    pub async fn message_count(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        status: &Option<SenderMailMessageStatus>,
        to_mail: &Option<String>,
    ) -> SenderResult<i64> {
        let mut sqlwhere = vec![];
        if let Some(s) = to_mail {
            sqlwhere.push(sql_format!("to_mail={}", s));
        }
        self.record
            .message_count(
                user_id,
                app_id,
                tpl_id,
                &status.map(|e| e as i8),
                if sqlwhere.is_empty() {
                    None
                } else {
                    Some(sqlwhere.join(" and "))
                },
            )
            .await
    }
    pub async fn message_list(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        status: &Option<SenderMailMessageStatus>,
        to_mail: &Option<String>,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<SenderMailMessageModel>> {
        let mut sqlwhere = vec![];
        if let Some(s) = to_mail {
            sqlwhere.push(sql_format!("to_mail={}", s));
        }
        self.record
            .message_list(
                user_id,
                app_id,
                tpl_id,
                &status.map(|e| e as i8),
                if sqlwhere.is_empty() {
                    None
                } else {
                    Some(sqlwhere.join(" and "))
                },
                page,
            )
            .await
    }
    pub async fn message_log_count(&self, message_id: &u64) -> SenderResult<i64> {
        self.logs.list_count(message_id).await
    }
    pub async fn message_log_list(
        &self,
        message_id: &u64,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<SenderLogModel>> {
        self.logs.list_data(message_id, page).await
    }

    //添加短信任务
    #[allow(clippy::too_many_arguments)]
    pub async fn add(
        &self,
        mail: &[String],
        app_id: &u64,
        tpl_id: &str,
        tpl_var: &str,
        expected_time: &u64,
        reply_mail: &Option<String>,
        user_id: &Option<u64>,
        cancel_key: &Option<String>,
    ) -> SenderResult<u64> {
        let user_id = user_id.unwrap_or_default();
        let add_time = now_time().unwrap_or_default();
        let reply_mail = reply_mail.to_owned().unwrap_or_default();
        let tpl_id = tpl_id.to_owned();
        let tpl_var = tpl_var.to_owned();
        let mut idata = Vec::with_capacity(mail.len());
        let add_data = mail
            .iter()
            .map(|e| (self.record.message_id(), e.to_owned()))
            .collect::<Vec<_>>();
        for (id, to) in add_data.iter() {
            #[allow(clippy::needless_update)]
            idata.push(sqlx_model::model_option_set!(SenderMailMessageModelRef,{
                id:id,
                app_id:*app_id,
                to_mail:to,
                reply_mail:reply_mail,
                tpl_id:tpl_id,
                tpl_var:tpl_var,
                try_num:0,
                status:SenderMailMessageStatus::Init as i8,
                add_time:add_time,
                send_time:0,
                user_id:user_id,
                expected_time:expected_time,
            }));
        }
        let mut tran = self.db.begin().await?;
        let row = Insert::<sqlx::MySql, SenderMailMessageModel, _>::new_vec(idata)
            .execute(&mut tran)
            .await?
            .rows_affected();
        if let Some(hk) = cancel_key {
            let ids = add_data.into_iter().map(|e| e.0).collect::<Vec<u64>>();
            let res = self.cancel.add(app_id, &ids, hk, Some(&mut tran)).await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        tran.commit().await?;
        Ok(row)
    }
    pub async fn cancel_form_message(
        &self,
        message: &SenderMailMessageModel,
        user_id: &u64,
    ) -> SenderResult<()> {
        let change = sqlx_model::model_option_set!(SenderMailMessageModelRef, {
            status: SenderMailMessageStatus::IsCancel as i8
        });
        Update::<MySql, SenderMailMessageModel, _>::new(change)
            .execute_by_pk(message, &self.db)
            .await?;
        self.logs
            .add_cancel_log(message.app_id, message.id, user_id)
            .await;
        Ok(())
    }
    //取消指定的数据
    pub async fn cancel_form_key(
        &self,
        smsc: &SenderKeyCancelModel,
        user_id: &u64,
    ) -> SenderResult<()> {
        let mut db = self.db.begin().await?;
        let res = self.cancel.cancel(smsc, user_id, Some(&mut db)).await;
        if res.is_err() {
            db.rollback().await?;
            return res.map(|_| ());
        }
        let change = sqlx_model::model_option_set!(SenderMailMessageModelRef, {
            status: SenderMailMessageStatus::IsCancel as i8
        });
        let res = Update::<MySql, SenderMailMessageModel, _>::new(change)
            .execute_by_where_call("id=?", |b, _| b.bind(smsc.message_id), &mut db)
            .await;
        if res.is_err() {
            db.rollback().await?;
            return res.map(|_| ()).map_err(|e| e.into());
        }
        db.commit().await?;
        self.logs
            .add_cancel_log(smsc.app_id, smsc.message_id, user_id)
            .await;
        Ok(())
    }
    //完成指定短信任务
    pub async fn finish(
        &self,
        event_type: String,
        channel: String,
        val: &SenderMailMessageModel,
        res: &Result<(), String>,
        try_num: u16,
    ) -> SenderResult<()> {
        let status = match res {
            Ok(()) => SenderMailMessageStatus::IsSend as i8,
            Err(_) => SenderMailMessageStatus::SendFail as i8,
        };
        let set_try_num = val.try_num + 1;
        let send_time = now_time().unwrap_or_default();
        self.logs
            .add_finish_log(event_type, val.app_id, val.id, channel, res)
            .await;
        let mut change = sqlx_model::model_option_set!(SenderMailMessageModelRef,{
            send_time:send_time,
            try_num:set_try_num
        });
        if SenderMailMessageStatus::IsSend.eq(status)
            || (SenderMailMessageStatus::SendFail.eq(status) && val.try_num + 1 >= try_num)
        {
            change.status = Some(&status);
        }
        Update::<MySql, SenderMailMessageModel, _>::new(change)
            .execute_by_pk(val, &self.db)
            .await?;
        Ok(())
    }
    pub async fn find_config_by_id(&self, id: &u64) -> SenderResult<SenderConfigModel> {
        self.config.find_by_id(id).await
    }
    pub async fn config_add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: SenderMailConfigType,
        config_data: Value,
        user_id: u64,
        add_user_id: u64,
    ) -> SenderResult<u64> {
        let config_data = match config_type {
            SenderMailConfigType::Limit => {
                macro_rules! param_get {
                    ($name:literal,$asfn:ident,$miss_err:literal,$wrong_err:literal) => {
                        match config_data.get($name) {
                            Some(val) => match val.$asfn() {
                                Some(val) => val,
                                None => return Err(SenderError::System($wrong_err.to_string())),
                            },
                            None => {
                                return Err(SenderError::System($miss_err.to_string()));
                            }
                        }
                    };
                }
                let range_time = param_get!(
                    "range_time",
                    as_u64,
                    "range time param miss ",
                    "range time param wrong "
                );
                let max_send = param_get!(
                    "max_send",
                    as_u64,
                    "range time param miss ",
                    "range time param wrong "
                ) as u32;
                match serde_json::to_string(&SenderMailConfigLimit {
                    range_time,
                    max_send,
                }) {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(SenderError::System(err.to_string()));
                    }
                }
            }
            SenderMailConfigType::Block => config_data.to_string(),
            SenderMailConfigType::BlockDomain => config_data.to_string(),
            SenderMailConfigType::Close => "".to_string(),
            SenderMailConfigType::PassTpl => config_data.to_string(),
            SenderMailConfigType::MaxOfSend => match config_data.as_u64() {
                Some(num) => (num as u32).to_string(),
                None => {
                    return Err(SenderError::System("send max need number".to_string()));
                }
            },
        };
        self.config
            .add(
                app_id,
                priority,
                config_type as i8,
                config_data,
                user_id,
                add_user_id,
            )
            .await
    }
    pub async fn config_del(&self, config: &SenderConfigModel, user_id: u64) -> SenderResult<u64> {
        self.config.del(config, user_id).await
    }
    pub async fn config_list(
        &self,
        user_id: Option<u64>,
        id: Option<u64>,
        app_id: Option<u64>,
    ) -> SenderResult<Vec<(SenderConfigModel, SenderMailConfigData)>> {
        let data = self.config.list_data(user_id, id, app_id).await?;
        Ok(data
            .into_iter()
            .map(|v| {
                let cd = match SenderMailConfigType::try_from(v.config_type) {
                    Ok(t) => match t {
                        SenderMailConfigType::Block => SenderMailConfigData::Block {
                            to: v.config_data.to_owned(),
                        },
                        SenderMailConfigType::BlockDomain => SenderMailConfigData::BlockDomain {
                            domain: v.config_data.to_owned(),
                        },
                        SenderMailConfigType::PassTpl => {
                            SenderMailConfigData::PassTpl(v.config_data.to_owned())
                        }
                        SenderMailConfigType::Close => SenderMailConfigData::Close,
                        SenderMailConfigType::MaxOfSend => match v.config_data.parse::<u32>() {
                            Ok(u) => SenderMailConfigData::MaxOfSend(u),
                            Err(_) => SenderMailConfigData::None,
                        },
                        SenderMailConfigType::Limit => {
                            match serde_json::from_slice::<SenderMailConfigLimit>(
                                v.config_data.as_bytes(),
                            ) {
                                Ok(t) => SenderMailConfigData::Limit(t),
                                Err(_) => SenderMailConfigData::None,
                            }
                        }
                    },
                    Err(_) => SenderMailConfigData::None,
                };
                (v, cd)
            })
            .collect::<Vec<_>>())
    }
    // //检测指定发送是否符合配置规则
    pub async fn send_check(
        &self,
        app_id: Option<u64>,
        tpl_id: &str,
        mails: &[String],
        send_time: u64,
    ) -> SenderResult<()> {
        if mails.is_empty() {
            return Err(SenderError::System("miss to mail box".to_string()));
        }
        let mut rule = self
            .config_list(None, None, Some(app_id.unwrap_or_default()))
            .await?;
        let mut limit_sql = vec![];
        let nowt = send_time;
        rule.sort_by(|a, b| a.0.priority.cmp(&b.0.priority));
        if let Some(max_send) = (|| {
            for t in rule.iter() {
                if let SenderMailConfigData::MaxOfSend(u) = t.1 {
                    return Some(u);
                }
            }
            None
        })() {
            return Err(SenderError::System(format!(
                "send mobile limit :{}",
                max_send
            )));
        }
        for (c, r) in rule.iter() {
            match r {
                SenderMailConfigData::Limit(limit) => {
                    if limit.range_time == 0 || limit.max_send == 0 || nowt < limit.range_time {
                        continue;
                    }
                    let msql = mails
                        .iter()
                        .map(|e| sql_format!("to_mail={}", e))
                        .collect::<Vec<String>>()
                        .join(" or ");
                    let stime = nowt - limit.range_time;
                    let sql = sql_format!(
                        "select count(*) as total,{} as limit_id,to from {}
                        where app_id={} and status={} and expected_time>={} and ({}) group by to",
                        c.id,
                        SenderMailMessageModel::table_name(),
                        c.app_id,
                        SenderMailMessageStatus::IsSend,
                        stime,
                        SqlExpr(msql)
                    );
                    limit_sql.push((sql, c.id, limit));
                }
                SenderMailConfigData::PassTpl(itpl_id) => {
                    if *tpl_id == *itpl_id {
                        break;
                    }
                }
                SenderMailConfigData::Block { to } => {
                    if mails.iter().any(|a| *a == *to) {
                        return Err(SenderError::System(format!(
                            "send block on :{} [{}]",
                            to, c.id
                        )));
                    }
                }
                SenderMailConfigData::BlockDomain { domain } => {
                    if mails
                        .iter()
                        .any(|a| a.split('@').nth(1).unwrap_or_default() == *domain)
                    {
                        return Err(SenderError::System(format!(
                            "send block on :{} [{}]",
                            domain, c.id
                        )));
                    }
                }
                SenderMailConfigData::Close => {
                    return Err(SenderError::System("send mail is close".to_string()));
                }
                _ => {}
            }
        }
        if !limit_sql.is_empty() {
            let sqls = limit_sql
                .iter()
                .map(|e| e.0.as_str())
                .collect::<Vec<&str>>()
                .join(" union all ");
            let data =
                sqlx::query_as::<_, (i64, i64, String)>(&format!("select * from ({}) as t", &sqls))
                    .fetch_all(&self.db)
                    .await?;
            for (_, id, limit) in limit_sql {
                if let Some(t) = data.iter().find(|e| e.1 as u64 == id) {
                    if t.0 >= limit.max_send.into() {
                        return Err(SenderError::System(format!(
                            "trigger limit rule :{} on {} [{}]",
                            limit.max_send, t.2, id
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct MailTaskAcquisitionRecord {
    records: Arc<MailTaskRecord>,
}
impl MailTaskAcquisitionRecord {
    pub fn new(records: Arc<MailTaskRecord>) -> Self {
        Self { records }
    }
}

#[async_trait]
impl TaskAcquisition<u64, MailTaskItem<()>> for MailTaskAcquisitionRecord {
    //复用父结构体方法实现
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> SenderResult<TaskRecord<u64, MailTaskItem<()>>> {
        MailTaskAcquisition::read_record(self, tasking_record, limit).await
    }
}

#[async_trait]
impl MailTaskAcquisition<()> for MailTaskAcquisitionRecord {
    //获取每个发送记录的关联记录数据，阿里云短信没用到，所以返回()
    async fn read_record_attr(
        &self,
        res: Vec<SenderMailMessageModel>,
    ) -> SenderResult<Vec<MailTaskItem<()>>> {
        Ok(res
            .into_iter()
            .map(|e| MailTaskItem { mail: e, attr: () })
            .collect())
    }
    //短信管理引用
    fn sms_record(&self) -> &MailTaskRecord {
        &self.records
    }
}
