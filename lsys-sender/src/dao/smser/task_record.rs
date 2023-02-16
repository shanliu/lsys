use std::{collections::HashMap, sync::Arc};

use crate::model::{
    SenderSmsCancelModel, SenderSmsCancelModelRef, SenderSmsCancelStatus, SenderSmsConfigData,
    SenderSmsConfigLimit, SenderSmsConfigModel, SenderSmsConfigModelRef, SenderSmsConfigStatus,
    SenderSmsConfigType, SenderSmsLogModel, SenderSmsLogModelRef, SenderSmsLogStatus,
    SenderSmsLogType, SenderSmsMessageModel, SenderSmsMessageModelRef, SenderSmsMessageStatus,
};
use lsys_core::{now_time, AppCore};

use parking_lot::Mutex;
use serde_json::Value;
use snowflake::SnowflakeIdGenerator;
use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Insert, ModelTableName, Select, SqlExpr, Update};

use tracing::warn;

use super::super::task::TaskValue;
use sqlx_model::SqlQuote;

//短信任务记录
#[derive(Clone)]
pub struct SmsTaskRecord {
    pub(crate) db: Pool<sqlx::MySql>,
    pub(crate) id_generator: Arc<Mutex<SnowflakeIdGenerator>>,
}

impl SmsTaskRecord {
    pub fn new(db: Pool<sqlx::MySql>, app_core: Arc<AppCore>) -> Self {
        let machine_id = app_core.config.get_int("snowflake_machine_id").unwrap_or(1);
        let node_id = app_core
            .config
            .get_int("snowflake_node_id")
            .unwrap_or_else(|_| {
                crc32fast::hash(
                    hostname::get()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .as_bytes(),
                )
                .into()
            });
        let id_generator = Arc::new(Mutex::new(SnowflakeIdGenerator::new(
            machine_id as i32,
            node_id as i32,
        )));
        Self { db, id_generator }
    }
    //读取短信任务数据
    pub async fn read(
        &self,
        tasking_record: &HashMap<u64, TaskValue>,
        limit: usize,
    ) -> Result<(Vec<SenderSmsMessageModel>, bool), sqlx::Error> {
        let mut sql_vec = vec![];
        sql_vec.push(sql_format!("status = {}", SenderSmsMessageStatus::Init));
        let ids = tasking_record.keys().copied().collect::<Vec<u64>>();
        if !ids.is_empty() {
            sql_vec.push(sql_format!(" id not in ({})", ids));
        }
        let mut app_res = Select::type_new::<SenderSmsMessageModel>()
            .fetch_all_by_where::<SenderSmsMessageModel, _>(
                &sqlx_model::WhereOption::Where(format!(
                    "{} order by id asc limit {}",
                    sql_vec.join(" and "),
                    limit + 1
                )),
                &self.db,
            )
            .await?;
        let next = if app_res.len() > limit {
            app_res.pop();
            true
        } else {
            false
        };
        Ok((app_res, next))
    }
    //添加短信任务
    pub async fn add(
        &self,
        mobiles: &[(String, String)],
        tpl_id: &str,
        tpl_var: &str,
        expected_time: &u64,
        user_id: &Option<u64>,
        cancel_key: &Option<String>,
    ) -> Result<u64, String> {
        let user_id = user_id.unwrap_or_default();
        let add_time = now_time().unwrap_or_default();
        let tpl_id = tpl_id.to_owned();
        let tpl_var = tpl_var.to_owned();
        let mut idata = Vec::with_capacity(mobiles.len());

        let add_data = mobiles
            .iter()
            .map(|e| {
                let id = self.id_generator.lock().real_time_generate() as u64;

                (id, &e.0, &e.1)
            })
            .collect::<Vec<_>>();
        for (id, area, mobile) in add_data.iter() {
            idata.push(sqlx_model::model_option_set!(SenderSmsMessageModelRef,{
                id:id,
                mobile:*mobile,
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
        let row = Insert::<sqlx::MySql, SenderSmsMessageModel, _>::new_vec(idata)
            .execute(&self.db)
            .await
            .map_err(|e| e.to_string())?
            .rows_affected();
        if let Some(hk) = cancel_key {
            if !hk.is_empty() {
                if hk.len() > 32 {
                    return Err("cancel key can't >32".to_owned());
                }
                let mut idata = Vec::with_capacity(mobiles.len());
                for (id, _, _) in add_data.iter() {
                    idata.push(sqlx_model::model_option_set!(SenderSmsCancelModelRef,{
                        sms_message_id:id,
                        cancel_hand:hk,
                        status:SenderSmsCancelStatus::Init as i8,
                        user_id:0,
                        cancel_time:add_time,
                    }));
                }
                Insert::<sqlx::MySql, SenderSmsCancelModel, _>::new_vec(idata)
                    .execute(&self.db)
                    .await
                    .map_err(|e| e.to_string())?;
            }
        }
        Ok(row)
    }

    //取消指定的数据
    pub async fn cancel_id(
        &self,
        smsc: &SenderSmsCancelModel,
        user_id: &u64,
    ) -> Result<(), String> {
        let mut db = self.db.begin().await.map_err(|e| e.to_string())?;
        let change = sqlx_model::model_option_set!(SenderSmsCancelModelRef,{
            status:SenderSmsCancelStatus::IsCancel as i8,
            user_id:user_id
        });
        let res = Update::<MySql, SenderSmsCancelModel, _>::new(change)
            .execute_by_pk(smsc, &mut db)
            .await
            .map_err(|e| e.to_string());
        if res.is_err() {
            db.rollback().await.map_err(|e| e.to_string())?;
            return res.map(|_| ());
        }
        let change = sqlx_model::model_option_set!(SenderSmsMessageModelRef, {
            status: SenderSmsMessageStatus::IsCancel as i8
        });
        let res = Update::<MySql, SenderSmsMessageModel, _>::new(change)
            .execute_by_where_call("id={}", |b, _| b.bind(smsc.sms_message_id), &mut db)
            .await
            .map_err(|e| e.to_string());
        if res.is_err() {
            db.rollback().await.map_err(|e| e.to_string())?;
            return res.map(|_| ());
        }
        db.commit().await.map_err(|e| e.to_string())?;
        let send_time = now_time().unwrap_or_default();
        let log_type = SenderSmsLogType::Cancel as i8;
        let send_type = "aliyun".to_string();
        let message = "cancal send".to_string();
        let idata = sqlx_model::model_option_set!(SenderSmsLogModelRef,{
            sms_message_id:smsc.sms_message_id,
            log_type:log_type,
            status: SenderSmsLogStatus::Fail as i8,
            send_type:send_type,
            message:message,
            create_time:send_time,
        });
        let tmp = Insert::<sqlx::MySql, SenderSmsLogModel, _>::new(idata)
            .execute(&self.db)
            .await;
        if let Err(ie) = tmp {
            warn!(
                "sms[{}] is cancel ,add history fail : {:?}",
                smsc.sms_message_id, ie
            );
        }
        Ok(())
    }
    //可取消发送的数据
    pub async fn cancel_data(&self, cancel_key: &str) -> Result<Vec<SenderSmsCancelModel>, String> {
        let status = SenderSmsCancelStatus::Init as i8;
        let cancel_key = cancel_key.to_owned();
        let rows = Select::type_new::<SenderSmsCancelModel>()
            .fetch_all_by_where_call::<SenderSmsCancelModel, _, _>(
                "cancel_hand ={} and status={}",
                |bind, _| bind.bind(cancel_key).bind(status),
                &self.db,
            )
            .await
            .map_err(|e| e.to_string())?;
        Ok(rows)
    }
    //完成指定短信任务
    pub async fn finish(
        &self,
        send_type: String,
        val: &SenderSmsMessageModel,
        res: &Result<(), String>,
        try_num: u16,
    ) -> Result<(), sqlx::Error> {
        let (status, log_status, err_msg) = match res {
            Ok(()) => (
                SenderSmsMessageStatus::IsSend as i8,
                SenderSmsLogStatus::Succ as i8,
                "".to_string(),
            ),
            Err(err) => (
                SenderSmsMessageStatus::SendFail as i8,
                SenderSmsLogStatus::Fail as i8,
                err.to_owned(),
            ),
        };
        let set_try_num = val.try_num + 1;
        let send_time = now_time().unwrap_or_default();
        let log_type = SenderSmsLogType::Send as i8;
        let idata = sqlx_model::model_option_set!(SenderSmsLogModelRef,{
            sms_message_id:val.id,
            log_type:log_type,
            status:log_status,
            send_type:send_type,
            message:err_msg,
            create_time:send_time,
        });

        let tmp = Insert::<sqlx::MySql, SenderSmsLogModel, _>::new(idata)
            .execute(&self.db)
            .await;
        if let Err(ie) = tmp {
            warn!("sms[{}] is send ,add history fail : {:?}", val.id, ie);
        }
        let mut change = sqlx_model::model_option_set!(SenderSmsMessageModelRef,{
            send_time:send_time,
            try_num:set_try_num
        });
        if SenderSmsMessageStatus::IsSend.eq(status)
            || (SenderSmsMessageStatus::SendFail.eq(status) && val.try_num >= try_num)
        {
            change.status = Some(&status);
        }
        Update::<MySql, SenderSmsMessageModel, _>::new(change)
            .execute_by_pk(val, &self.db)
            .await?;
        Ok(())
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_config_by_id,
        u64,
        SenderSmsConfigModel,
        Result<SenderSmsConfigModel,sqlx::Error>,
        id,
        "id={id}"
    );
    pub async fn config_add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: SenderSmsConfigType,
        config_data: Value,
        user_id: u64,
    ) -> Result<u64, String> {
        let app_id = app_id.unwrap_or_default();
        let time = now_time().unwrap_or_default();
        let config_data = match config_type {
            SenderSmsConfigType::Limit => {
                macro_rules! param_get {
                    ($name:literal,$asfn:ident,$miss_err:literal,$wrong_err:literal) => {
                        match config_data.get($name) {
                            Some(val) => match val.$asfn() {
                                Some(val) => val,
                                None => return Err($wrong_err.to_string()),
                            },
                            None => {
                                return Err($miss_err.to_string());
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
                        return Err(err.to_string());
                    }
                }
            }
            SenderSmsConfigType::Block => "".to_string(),
            SenderSmsConfigType::Close => "".to_string(),
            SenderSmsConfigType::PassTpl => config_data.to_string(),
            SenderSmsConfigType::MaxOfSend => match config_data.as_u64() {
                Some(num) => (num as u32).to_string(),
                None => {
                    return Err("send max need number".to_string());
                }
            },
        };
        let config_type = config_type as i8;
        let add = sqlx_model::model_option_set!(SenderSmsConfigModelRef, {
            app_id:app_id,
            priority:priority,
            config_type:config_type,
            user_id:user_id,
            add_time:time,
            status:SenderSmsConfigStatus::Enable as i8,
            config_data:config_data,
        });
        Insert::<sqlx::MySql, SenderSmsConfigModel, _>::new(add)
            .execute(&self.db)
            .await
            .map(|e| e.last_insert_id())
            .map_err(|e| e.to_string())
    }
    pub async fn config_del(
        &self,
        config: &SenderSmsConfigModel,
        user_id: u64,
    ) -> Result<u64, String> {
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SenderSmsConfigModelRef,{
            status:SenderSmsConfigStatus::Delete as i8,
            delete_time:time,
            delete_user_id:user_id
        });
        let res = Update::<sqlx::MySql, SenderSmsConfigModel, _>::new(change)
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
    pub async fn config_list(
        &self,
        app_id: Option<u64>,
    ) -> Result<Vec<(SenderSmsConfigModel, SenderSmsConfigData)>, sqlx::Error> {
        let app_id = app_id.unwrap_or_default();
        let sql = sql_format!(
            "app_id = {} and status ={} order by id desc",
            app_id,
            SenderSmsConfigStatus::Enable
        );
        Select::type_new::<SenderSmsConfigModel>()
            .fetch_all_by_where::<SenderSmsConfigModel, _>(
                &sqlx_model::WhereOption::Where(sql),
                &self.db,
            )
            .await
            .map(|e| {
                e.into_iter()
                    .map(|v| {
                        let cd = match SenderSmsConfigType::try_from(v.config_type) {
                            Ok(t) => match t {
                                SenderSmsConfigType::Block => {
                                    let mut split = v.config_data.split('-');
                                    SenderSmsConfigData::Block(
                                        split.next().unwrap_or("").to_owned(),
                                        split.next().unwrap_or("").to_owned(),
                                    )
                                }
                                SenderSmsConfigType::PassTpl => {
                                    SenderSmsConfigData::PassTpl(v.config_data.to_owned())
                                }
                                SenderSmsConfigType::Close => SenderSmsConfigData::Close,
                                SenderSmsConfigType::MaxOfSend => {
                                    match v.config_data.parse::<u32>() {
                                        Ok(u) => SenderSmsConfigData::MaxOfSend(u),
                                        Err(_) => SenderSmsConfigData::None,
                                    }
                                }
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
                    .collect::<Vec<_>>()
            })
    }
    //检测指定发送是否符合配置规则
    pub async fn send_check(
        &self,
        app_id: Option<u64>,
        tpl_id: &str,
        mobiles: &[(String, String)],
        send_time: u64,
    ) -> Result<(), String> {
        if mobiles.is_empty() {
            return Err("miss mobile".to_string());
        }
        let mut rule = self.config_list(app_id).await.map_err(|e| e.to_string())?;
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
            return Err(format!("send mobile limit :{}", max_send));
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
                        where app_id={} and log_type={} and status={} and expected_time>={} and ({}) group by area,mobile",
                        c.id,
                        SenderSmsMessageModel::table_name(),
                        c.app_id,
                        SenderSmsLogType::Send,
                        SenderSmsLogStatus::Succ,
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
                SenderSmsConfigData::Block(area, mobile) => {
                    if mobiles.iter().any(|a| *a.0 == *area && *a.1 == *mobile) {
                        return Err(format!("send block on :{}{} [{}]", area, mobile, c.id));
                    }
                }
                SenderSmsConfigData::Close => {
                    return Err("send sms is close".to_string());
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
            let data = sqlx::query_as::<_, (i64, u64, String, String)>(&format!(
                "select * from ({}) as t",
                &sqls
            ))
            .fetch_all(&self.db)
            .await
            .map_err(|e| e.to_string())?;
            for (_, id, limit) in limit_sql {
                if let Some(t) = data.iter().find(|e| e.1 == id) {
                    if t.0 > limit.max_send.into() {
                        return Err(format!(
                            "send sms limit :{} on {}{} [{}]",
                            limit.max_send, t.2, t.3, id
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
