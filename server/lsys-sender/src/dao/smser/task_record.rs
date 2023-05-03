use std::{collections::HashMap, sync::Arc};

use crate::{
    dao::{
        MessageCancel, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
        TaskAcquisition, TaskRecord,
    },
    model::{
        SenderConfigModel, SenderKeyCancelModel, SenderLogModel, SenderSmsConfigData,
        SenderSmsConfigLimit, SenderSmsConfigType, SenderSmsMessageModel, SenderSmsMessageModelRef,
        SenderSmsMessageStatus, SenderType,
    },
};
use async_trait::async_trait;
use lsys_core::{now_time, AppCore, FluentMessage, LimitParam, PageParam, RequestEnv};

use lsys_logger::dao::ChangeLogger;
use serde_json::Value;
use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Insert, ModelTableName, SqlExpr, Update};

use super::{SmsTaskAcquisition, SmsTaskItem, TaskData};
use sqlx_model::SqlQuote;

//短信任务记录

pub struct SmsTaskRecord {
    pub(crate) db: Pool<sqlx::MySql>,
    pub(crate) config: SenderConfig,
    pub(crate) cancel: MessageCancel,
    pub(crate) msg_logs: MessageLogs,
    pub(crate) record: MessageReader<SenderSmsMessageModel>,
}

impl SmsTaskRecord {
    pub fn new(
        db: Pool<sqlx::MySql>,
        app_core: Arc<AppCore>,
        fluent: Arc<FluentMessage>,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            config: SenderConfig::new(db.clone(), logger, SenderType::Smser),
            cancel: MessageCancel::new(db.clone(), SenderType::Smser),
            msg_logs: MessageLogs::new(db.clone(), SenderType::Smser),
            record: MessageReader::new(db.clone(), app_core, fluent),
            db,
        }
    }
    //读取短信任务数据
    pub async fn read(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> SenderResult<(Vec<SenderSmsMessageModel>, bool)> {
        self.record
            .read(tasking_record, SenderSmsMessageStatus::Init as i8, limit)
            .await
    }
    //根据ID获取消息
    pub async fn find_message_by_id(&self, id: &u64) -> SenderResult<SenderSmsMessageModel> {
        self.record.find_message_by_id(id).await
    }
    //消息数量
    pub async fn message_count(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        status: &Option<SenderSmsMessageStatus>,
        mobile: &Option<String>,
    ) -> SenderResult<i64> {
        let mut sqlwhere = vec![];
        if let Some(s) = mobile {
            sqlwhere.push(sql_format!("mobile={}", s));
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
    //消息列表
    pub async fn message_list(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        status: &Option<SenderSmsMessageStatus>,
        mobile: &Option<String>,
        limit: &Option<LimitParam>,
    ) -> SenderResult<(Vec<SenderSmsMessageModel>, Option<u64>)> {
        let mut sqlwhere = vec![];
        if let Some(s) = mobile {
            sqlwhere.push(sql_format!("mobile={}", s));
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
                limit,
            )
            .await
            .map(|(e, n)| (e, n.map(|t| t.id)))
    }
    //消息日志数量
    pub async fn message_log_count(&self, message_id: &u64) -> SenderResult<i64> {
        self.msg_logs.list_count(message_id).await
    }
    //消息日志列表
    pub async fn message_log_list(
        &self,
        message_id: &u64,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<SenderLogModel>> {
        self.msg_logs.list_data(message_id, page).await
    }
    //添加短信任务
    #[allow(clippy::too_many_arguments)]
    pub async fn add(
        &self,
        mobiles: &[(String, String)],
        app_id: &u64,
        tpl_id: &str,
        tpl_var: &str,
        expected_time: &u64,
        user_id: &Option<u64>,
        cancel_key: &Option<String>,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let user_id = user_id.unwrap_or_default();
        let add_time = now_time().unwrap_or_default();
        let tpl_id = tpl_id.to_owned();
        let tpl_var = tpl_var.to_owned();
        let mut idata = Vec::with_capacity(mobiles.len());

        let add_data = mobiles
            .iter()
            .map(|e| (self.record.message_id(), &e.0, &e.1))
            .collect::<Vec<_>>();
        for (id, area, mobile) in add_data.iter() {
            #[allow(clippy::needless_update)]
            idata.push(sqlx_model::model_option_set!(SenderSmsMessageModelRef,{
                id:id,
                mobile:*mobile,
                app_id:*app_id,
                area:*area,
                tpl_id:tpl_id,
                tpl_var:tpl_var,
                try_num:0,
                status:SenderSmsMessageStatus::Init as i8,
                add_time:add_time,
                send_time:0,
                user_id:user_id,
                expected_time:expected_time,
            }));
        }
        let mut tran = self.db.begin().await?;
        let row = Insert::<sqlx::MySql, SenderSmsMessageModel, _>::new_vec(idata)
            .execute(&mut tran)
            .await?
            .rows_affected();
        let ids = add_data.into_iter().map(|e| e.0).collect::<Vec<u64>>();
        if let Some(hk) = cancel_key {
            let res = self.cancel.add(app_id, &ids, hk, Some(&mut tran)).await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        tran.commit().await?;

        self.msg_logs.add_init_log(app_id, &ids, env_data).await;

        Ok(row)
    }
    //取消短信发送
    pub async fn cancel_form_message(
        &self,
        message: &SenderSmsMessageModel,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<()> {
        let change = sqlx_model::model_option_set!(SenderSmsMessageModelRef, {
            status: SenderSmsMessageStatus::IsCancel as i8
        });
        Update::<MySql, SenderSmsMessageModel, _>::new(change)
            .execute_by_pk(message, &self.db)
            .await?;
        self.msg_logs
            .add_cancel_log(message.app_id, message.id, user_id, env_data)
            .await;
        Ok(())
    }
    //通过KEY取消短信发送
    pub async fn cancel_form_key(
        &self,
        smsc: &SenderKeyCancelModel,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<()> {
        let mut db = self.db.begin().await?;
        let res = self.cancel.cancel(smsc, user_id, Some(&mut db)).await;
        if res.is_err() {
            db.rollback().await?;
            return res.map(|_| ());
        }
        let change = sqlx_model::model_option_set!(SenderSmsMessageModelRef, {
            status: SenderSmsMessageStatus::IsCancel as i8
        });
        let res = Update::<MySql, SenderSmsMessageModel, _>::new(change)
            .execute_by_where_call("id=?", |b, _| b.bind(smsc.message_id), &mut db)
            .await;
        if res.is_err() {
            db.rollback().await?;
            return res.map(|_| ()).map_err(|e| e.into());
        }
        db.commit().await?;
        self.msg_logs
            .add_cancel_log(smsc.app_id, smsc.message_id, user_id, env_data)
            .await;
        Ok(())
    }
    //完成指定短信任务回调
    pub async fn finish(
        &self,
        event_type: String,
        channel: String,
        val: &SenderSmsMessageModel,
        res: &Result<(), String>,
        try_num: u16,
    ) -> SenderResult<()> {
        let status = match res {
            Ok(()) => SenderSmsMessageStatus::IsSend as i8,
            Err(_) => SenderSmsMessageStatus::SendFail as i8,
        };
        let set_try_num = val.try_num + 1;
        let send_time = now_time().unwrap_or_default();
        self.msg_logs
            .add_finish_log(event_type, val.app_id, val.id, channel, res)
            .await;
        let mut change = sqlx_model::model_option_set!(SenderSmsMessageModelRef,{
            send_time:send_time,
            try_num:set_try_num
        });
        if SenderSmsMessageStatus::IsSend.eq(status)
            || (SenderSmsMessageStatus::SendFail.eq(status) && val.try_num + 1 >= try_num)
        {
            change.status = Some(&status);
        }
        Update::<MySql, SenderSmsMessageModel, _>::new(change)
            .execute_by_pk(val, &self.db)
            .await?;
        Ok(())
    }
    //查找短信基本配置
    pub async fn find_config_by_id(&self, id: &u64) -> SenderResult<SenderConfigModel> {
        self.config.find_by_id(id).await
    }
    //短信配置添加
    #[allow(clippy::too_many_arguments)]
    pub async fn config_add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: SenderSmsConfigType,
        config_data: Value,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        let config_data = match config_type {
            SenderSmsConfigType::Limit => {
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
                match serde_json::to_string(&SenderSmsConfigLimit {
                    range_time,
                    max_send,
                }) {
                    Ok(val) => val,
                    Err(err) => {
                        return Err(SenderError::System(err.to_string()));
                    }
                }
            }
            SenderSmsConfigType::Block => config_data.to_string(),
            SenderSmsConfigType::Close => "".to_string(),
            SenderSmsConfigType::PassTpl => config_data.to_string(),
            SenderSmsConfigType::MaxOfSend => match config_data.as_u64() {
                Some(num) => (num as u32).to_string(),
                None => {
                    return Err(SenderError::System("send max need number".to_string()));
                }
            },
        };
        let id = self
            .config
            .add(
                app_id,
                priority,
                config_type as i8,
                config_data,
                user_id,
                add_user_id,
                env_data,
            )
            .await?;
        Ok(id)
    }
    //短信配置删除
    pub async fn config_del(
        &self,
        config: &SenderConfigModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.config.del(config, user_id, env_data).await
    }
    //短信配置列表数据
    pub async fn config_list(
        &self,
        user_id: Option<u64>,
        id: Option<u64>,
        app_id: Option<u64>,
    ) -> SenderResult<Vec<(SenderConfigModel, SenderSmsConfigData)>> {
        let data = self.config.list_data(user_id, id, app_id).await?;
        Ok(data
            .into_iter()
            .map(|v| {
                let cd = match SenderSmsConfigType::try_from(v.config_type) {
                    Ok(t) => match t {
                        SenderSmsConfigType::Block => {
                            let mut split = v.config_data.split('-');
                            SenderSmsConfigData::Block {
                                area: split.next().unwrap_or("").to_owned(),
                                mobile: split.next().unwrap_or("").to_owned(),
                            }
                        }
                        SenderSmsConfigType::PassTpl => {
                            SenderSmsConfigData::PassTpl(v.config_data.to_owned())
                        }
                        SenderSmsConfigType::Close => SenderSmsConfigData::Close,
                        SenderSmsConfigType::MaxOfSend => match v.config_data.parse::<u32>() {
                            Ok(u) => SenderSmsConfigData::MaxOfSend(u),
                            Err(_) => SenderSmsConfigData::None,
                        },
                        SenderSmsConfigType::Limit => {
                            match serde_json::from_slice::<SenderSmsConfigLimit>(
                                v.config_data.as_bytes(),
                            ) {
                                Ok(t) => SenderSmsConfigData::Limit(t),
                                Err(_) => SenderSmsConfigData::None,
                            }
                        }
                    },
                    Err(_) => SenderSmsConfigData::None,
                };
                (v, cd)
            })
            .collect::<Vec<_>>())
    }
    //检测指定发送是否符合配置规则
    pub async fn send_check(
        &self,
        app_id: Option<u64>,
        tpl_id: &str,
        mobiles: &[(String, String)],
        send_time: u64,
    ) -> SenderResult<()> {
        if mobiles.is_empty() {
            return Err(SenderError::System("miss mobile".to_string()));
        }
        let mut rule = self
            .config_list(None, None, Some(app_id.unwrap_or_default()))
            .await?;
        let mut limit_sql = vec![];
        let nowt = send_time;
        rule.sort_by(|a, b| a.0.priority.cmp(&b.0.priority));
        if let Some(max_send) = (|| {
            for t in rule.iter() {
                if let SenderSmsConfigData::MaxOfSend(u) = t.1 {
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
                SenderSmsConfigData::Limit(limit) => {
                    if limit.range_time == 0 || limit.max_send == 0 || nowt < limit.range_time {
                        continue;
                    }
                    let msql = mobiles
                        .iter()
                        .map(|e| sql_format!("area={} and mobile={}", e.0, e.1))
                        .collect::<Vec<String>>()
                        .join(" or ");
                    let stime = nowt - limit.range_time;
                    let sql = sql_format!(
                        "select count(*) as total,{} as limit_id,area,mobile from {}
                        where app_id={} and status={} and expected_time>={} and ({}) group by area,mobile",
                        c.id,
                        SenderSmsMessageModel::table_name(),
                        c.app_id,
                        SenderSmsMessageStatus::IsSend,
                        stime,
                        SqlExpr(msql)
                    );
                    limit_sql.push((sql, c.id, limit));
                }
                SenderSmsConfigData::PassTpl(itpl_id) => {
                    if *tpl_id == *itpl_id {
                        break;
                    }
                }
                SenderSmsConfigData::Block { area, mobile } => {
                    if mobiles.iter().any(|a| *a.0 == *area && *a.1 == *mobile) {
                        return Err(SenderError::System(format!(
                            "send block on :{}{} [{}]",
                            area, mobile, c.id
                        )));
                    }
                }
                SenderSmsConfigData::Close => {
                    return Err(SenderError::System("send sms is close".to_string()));
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
            let data = sqlx::query_as::<_, (i64, i64, String, String)>(&format!(
                "select * from ({}) as t",
                &sqls
            ))
            .fetch_all(&self.db)
            .await?;
            for (_, id, limit) in limit_sql {
                if let Some(t) = data.iter().find(|e| e.1 as u64 == id) {
                    if t.0 >= limit.max_send.into() {
                        return Err(SenderError::System(format!(
                            "trigger limit rule :{} on {}{} [{}]",
                            limit.max_send, t.2, t.3, id
                        )));
                    }
                }
            }
        }
        Ok(())
    }
}

pub struct SmsTaskAcquisitionRecord {
    records: Arc<SmsTaskRecord>,
}
impl SmsTaskAcquisitionRecord {
    pub fn new(records: Arc<SmsTaskRecord>) -> Self {
        Self { records }
    }
}

#[async_trait]
impl TaskAcquisition<u64, SmsTaskItem<()>> for SmsTaskAcquisitionRecord {
    //复用父结构体方法实现
    async fn read_record(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> SenderResult<TaskRecord<u64, SmsTaskItem<()>>> {
        SmsTaskAcquisition::read_record(self, tasking_record, limit).await
    }
}

#[async_trait]
impl SmsTaskAcquisition<()> for SmsTaskAcquisitionRecord {
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
