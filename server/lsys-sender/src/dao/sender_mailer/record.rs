use std::{collections::HashSet, sync::Arc};

use crate::{
    dao::{
        logger::LogMessage, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
    },
    model::{
        SenderConfigModel, SenderLogModel, SenderMailBodyModel, SenderMailBodyModelRef,
        SenderMailBodyStatus, SenderMailConfigData, SenderMailConfigLimit, SenderMailConfigType,
        SenderMailMessageModel, SenderMailMessageModelRef, SenderMailMessageStatus, SenderType,
    },
};
use lsys_core::{now_time, LimitParam, PageParam, RequestEnv};

use lsys_logger::dao::ChangeLogger;
use serde_json::Value;
use sqlx::{MySql, Pool};
use sqlx_model::{sql_format, Insert, ModelTableName, SqlExpr, Update};

use sqlx_model::SqlQuote;

//短信任务记录

pub struct MailRecord {
    db: Pool<sqlx::MySql>,
    config: Arc<SenderConfig>,
    logger: Arc<ChangeLogger>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
}

impl MailRecord {
    pub fn new(
        db: Pool<sqlx::MySql>,
        config: Arc<SenderConfig>,
        logger: Arc<ChangeLogger>,
        message_logs: Arc<MessageLogs>,
        message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
    ) -> Self {
        Self {
            config,
            logger,
            message_logs,
            message_reader,
            db,
        }
    }
    pub async fn find_message_by_id(&self, id: &u64) -> SenderResult<SenderMailMessageModel> {
        self.message_reader.find_message_by_id(id).await
    }
    pub async fn find_body_by_id(&self, id: &u64) -> SenderResult<SenderMailBodyModel> {
        self.message_reader.find_body_by_id(id).await
    }
    pub async fn message_count(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        body_id: &Option<u64>,
        status: &Option<SenderMailMessageStatus>,
        to_mail: &Option<String>,
    ) -> SenderResult<i64> {
        let mut sqlwhere = vec![];
        if let Some(s) = to_mail {
            sqlwhere.push(sql_format!("m.to_mail={}", s));
        }

        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("b.app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("b.user_id={} ", uid));
        }
        if let Some(t) = tpl_id {
            sqlwhere.push(sql_format!("b.tpl_id={} ", t));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("m.status={} ", *s));
        }
        if let Some(s) = body_id {
            sqlwhere.push(sql_format!("m.sender_body_id={} ", *s));
        }
        let sql = sql_format!(
            "select count(*) as total from {} as m join {} as b on m.sender_body_id=b.id {}",
            SenderMailMessageModel::table_name(),
            SenderMailBodyModel::table_name(),
            SqlExpr(if sqlwhere.is_empty() {
                "".to_string()
            } else {
                format!("where {}", sqlwhere.join(" and "))
            })
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn message_list(
        &self,
        user_id: &Option<u64>,
        app_id: &Option<u64>,
        tpl_id: &Option<String>,
        body_id: &Option<u64>,
        status: &Option<SenderMailMessageStatus>,
        to_mail: &Option<String>,
        limit: &Option<LimitParam>,
    ) -> SenderResult<(
        Vec<(SenderMailMessageModel, Option<SenderMailBodyModel>)>,
        Option<u64>,
    )> {
        let mut sqlwhere = vec![];
        if let Some(s) = to_mail {
            sqlwhere.push(sql_format!("m.to_mail={}", s));
        }

        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("b.app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("b.user_id={} ", uid));
        }
        if let Some(t) = tpl_id {
            sqlwhere.push(sql_format!("b.tpl_id={} ", t));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("m.status={} ", *s));
        }
        if let Some(s) = body_id {
            sqlwhere.push(sql_format!("m.sender_body_id={} ", *s));
        }

        let where_sql = if let Some(page) = limit {
            let page_where = page.where_sql(
                "m.id",
                if sqlwhere.is_empty() {
                    None
                } else {
                    Some("and")
                },
            );
            format!(
                "{} {} {} order by {} {} ",
                if !sqlwhere.is_empty() || !page_where.is_empty() {
                    "where "
                } else {
                    ""
                },
                sqlwhere.join(" and "),
                page_where,
                page.order_sql("m.id"),
                page.limit_sql(),
            )
        } else {
            format!(
                "{} {}  order by id desc",
                if sqlwhere.is_empty() { "where " } else { "" },
                sqlwhere.join(" and ")
            )
        };

        let sql = sql_format!(
            "select m.* from {} as m join {} as b on m.sender_body_id=b.id {}",
            SenderMailMessageModel::table_name(),
            SenderMailBodyModel::table_name(),
            SqlExpr(where_sql)
        );

        let res = sqlx::query_as::<_, SenderMailMessageModel>(sql.as_str());
        let mut m_data = res.fetch_all(&self.db).await?;

        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut m_data))
            .unwrap_or_default();

        let pks = m_data
            .iter()
            .map(|t| t.sender_body_id)
            .collect::<HashSet<u64>>()
            .into_iter()
            .collect::<Vec<u64>>();

        let b_data = if !pks.is_empty() {
            let sql = sql_format!(
                "select * from {} where id in ({})",
                SenderMailBodyModel::table_name(),
                pks
            );
            let res = sqlx::query_as::<_, SenderMailBodyModel>(&sql);
            res.fetch_all(&self.db).await?
        } else {
            vec![]
        };

        let out_data = m_data
            .into_iter()
            .map(|e| {
                let tmp = b_data
                    .iter()
                    .find(|t| t.id == e.sender_body_id)
                    .map(|s| s.to_owned());
                (e, tmp)
            })
            .collect::<Vec<_>>();

        Ok((out_data, next.map(|t| t.id)))
    }
    pub async fn message_log_count(&self, message_id: &u64) -> SenderResult<i64> {
        self.message_logs.list_count(message_id).await
    }
    pub async fn message_log_list(
        &self,
        message_id: &u64,
        page: &Option<PageParam>,
    ) -> SenderResult<Vec<SenderLogModel>> {
        self.message_logs.list_data(message_id, page).await
    }

    //添加短信任务
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn add<'t>(
        &self,
        mail: &[&'t str],
        app_id: &u64,
        tpl_id: &str,
        tpl_var: &str,
        expected_time: &u64,
        reply_mail: &Option<String>,
        user_id: &Option<u64>,
        max_try_num: &Option<u8>,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(u64, Vec<(u64, &'t str)>)> {
        let user_id = user_id.unwrap_or_default();
        let add_time = now_time().unwrap_or_default();
        let reply_mail = reply_mail.to_owned().unwrap_or_default();
        let tpl_id = tpl_id.to_owned();
        let tpl_var = tpl_var.to_owned();
        let mut max_try_num = max_try_num.unwrap_or(1);
        if max_try_num == 0 {
            max_try_num = 1
        }
        if max_try_num > 10 {
            max_try_num = 10
        }
        let mut idata = Vec::with_capacity(mail.len());
        let reqid = env_data
            .map(|t| t.request_ip.to_owned().unwrap_or_default())
            .unwrap_or_default();
        let add_data = mail
            .iter()
            .map(|e| {
                let id = self.message_reader.message_id();
                (id, *e, e.to_string())
            })
            .collect::<Vec<_>>();
        let mut tran = self.db.begin().await?;

        let user_ip = env_data
            .map(|e| e.request_ip.clone().unwrap_or_default())
            .unwrap_or_default();

        let msg_id = Insert::<sqlx::MySql, SenderMailBodyModel, _>::new(
            sqlx_model::model_option_set!(SenderMailBodyModelRef,{
                request_id:reqid,
                app_id:*app_id,
                tpl_id:tpl_id,
                tpl_var:tpl_var,
                status:SenderMailBodyStatus::Init as i8,
                add_time:add_time,
                reply_mail:reply_mail,
                max_try_num:max_try_num as u16,
                user_id:user_id,
                user_ip:user_ip,
                expected_time:expected_time,
            }),
        )
        .execute(&mut tran)
        .await?
        .last_insert_id();
        let res_data = "".to_string();
        for (aid, _, to) in add_data.iter() {
            idata.push(sqlx_model::model_option_set!(SenderMailMessageModelRef,{
                id:aid,
                sender_body_id:msg_id,
                to_mail:to,
                try_num:0,
                status:SenderMailMessageStatus::Init as i8,
                send_time:0,
                res_data:res_data,
            }));
        }

        let row = Insert::<sqlx::MySql, SenderMailMessageModel, _>::new_vec(idata)
            .execute(&mut tran)
            .await?
            .rows_affected();

        tran.commit().await?;

        self.logger
            .add(
                &LogMessage {
                    action: "add",
                    body_id: msg_id,
                    message_id: None,
                    sender_type: SenderType::Mailer as i8,
                },
                &Some(msg_id),
                &Some(user_id),
                &Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok((
            row,
            add_data.into_iter().map(|e| (e.0, e.1)).collect::<Vec<_>>(),
        ))
    }
    pub(crate) async fn cancel_form_message(
        &self,
        body: &SenderMailBodyModel,
        message: &SenderMailMessageModel,
        user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<()> {
        if SenderMailMessageStatus::IsCancel.eq(message.status) {
            return Ok(());
        }
        if SenderMailMessageStatus::Init.eq(message.status) {
            let change = sqlx_model::model_option_set!(SenderMailMessageModelRef, {
                status: SenderMailMessageStatus::IsCancel as i8
            });
            Update::<MySql, SenderMailMessageModel, _>::new(change)
                .execute_by_pk(message, &self.db)
                .await?;

            self.logger
                .add(
                    &LogMessage {
                        action: "cancel",
                        body_id: body.id,
                        message_id: Some(message.id),
                        sender_type: SenderType::Mailer as i8,
                    },
                    &Some(message.id),
                    &Some(*user_id),
                    &Some(*user_id),
                    None,
                    env_data,
                )
                .await;

            return Ok(());
        }
        Err(SenderError::System(format!(
            "can't be cancel,status:{}",
            message.status
        )))
    }
    pub async fn find_config_by_id(&self, id: &u64) -> SenderResult<SenderConfigModel> {
        self.config.find_by_id(id).await
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn config_add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: SenderMailConfigType,
        config_data: Value,
        user_id: u64,
        add_user_id: u64,
        env_data: Option<&RequestEnv>,
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
                env_data,
            )
            .await
    }
    pub async fn config_del(
        &self,
        config: &SenderConfigModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<u64> {
        self.config.del(config, user_id, env_data).await
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
        mails: &[&str],
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
                "send mail limit :{}",
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
                        "select count(*) as total,{} as limit_id,to_mail from {}
                        where app_id={} and status in ({}) and expected_time>={} and ({}) group by to_mail",
                        c.id,
                        SenderMailBodyModel::table_name(),
                        c.app_id,
                        &[SenderMailMessageStatus::IsSend as i8,SenderMailMessageStatus::IsReceived as i8],
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
