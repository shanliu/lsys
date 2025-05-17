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
use lsys_core::{
    fluent_message, now_time, string_clear, valid_key, LimitParam, PageParam, RequestEnv,
    StringClear, ValidEmail, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
    STRING_CLEAR_FORMAT,
};

use lsys_core::db::{Insert, ModelTableName, SqlExpr, Update};
use lsys_core::sql_format;
use lsys_logger::dao::ChangeLoggerDao;
use serde_json::Value;
use sqlx::Pool;

use lsys_core::db::SqlQuote;

//短信任务记录

pub struct MailRecord {
    db: Pool<sqlx::MySql>,
    config: Arc<SenderConfig>,
    logger: Arc<ChangeLoggerDao>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
}

impl MailRecord {
    pub fn new(
        db: Pool<sqlx::MySql>,
        config: Arc<SenderConfig>,
        logger: Arc<ChangeLoggerDao>,
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
    pub async fn find_message_by_id(&self, id: u64) -> SenderResult<SenderMailMessageModel> {
        self.message_reader.find_message_by_id(&id).await
    }
    pub async fn find_body_by_id(&self, id: u64) -> SenderResult<SenderMailBodyModel> {
        self.message_reader.find_body_by_id(&id).await
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn message_count(
        &self,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_id: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<SenderMailMessageStatus>,
        to_mail: Option<&str>,
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
            let t = string_clear(t, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if t.is_empty() {
                return Ok(0);
            }
            sqlwhere.push(sql_format!("b.tpl_id={} ", t));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("m.status={} ", s));
        }
        if let Some(s) = body_id {
            sqlwhere.push(sql_format!("m.sender_body_id={} ", s));
        }
        if let Some(s) = msg_snid {
            sqlwhere.push(sql_format!("m.snid={} ", s));
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
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_id: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<SenderMailMessageStatus>,
        to_mail: Option<&str>,
        limit: Option<&LimitParam>,
    ) -> SenderResult<(
        Vec<(SenderMailMessageModel, Option<SenderMailBodyModel>)>,
        Option<u64>,
    )> {
        let mut sqlwhere = vec![];
        if let Some(s) = to_mail {
            let s = string_clear(s, StringClear::Option(STRING_CLEAR_FORMAT), Some(255));
            if s.is_empty() {
                return Ok((vec![], Some(0)));
            }
            sqlwhere.push(sql_format!("m.to_mail={}", s));
        }

        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("b.app_id = {}  ", aid));
        }
        if let Some(uid) = user_id {
            sqlwhere.push(sql_format!("b.user_id={} ", uid));
        }
        if let Some(t) = tpl_id {
            let t = string_clear(t, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if t.is_empty() {
                return Ok((vec![], Some(0)));
            }
            sqlwhere.push(sql_format!("b.tpl_id={} ", t));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("m.status={} ", s));
        }
        if let Some(s) = body_id {
            sqlwhere.push(sql_format!("m.sender_body_id={} ", s));
        }
        if let Some(s) = msg_snid {
            sqlwhere.push(sql_format!("m.snid={} ", s));
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
                "{} {}  order by m.id desc",
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
    pub async fn message_log_count(&self, message_id: u64) -> SenderResult<i64> {
        self.message_logs.list_count(message_id).await
    }
    pub async fn message_log_list(
        &self,
        message_id: u64,
        page: Option<&PageParam>,
    ) -> SenderResult<Vec<SenderLogModel>> {
        self.message_logs.list_data(message_id, page).await
    }
    #[allow(clippy::too_many_arguments)]
    async fn add_param_valid(
        &self,
        mail: &[&str],
        app_id: u64,
        tpl_id: &str,
        tpl_var: &str,
        reply_mail: Option<&str>,
        max_try_num: u8,
    ) -> SenderResult<()> {
        let mut param_valid = ValidParam::default();
        for mt in mail {
            param_valid.add(
                valid_key!("mail"),
                mt,
                &ValidParamCheck::default().add_rule(ValidEmail::default()),
            );
        }
        if let Some(tmp) = reply_mail {
            param_valid.add(
                valid_key!("reply_mail"),
                &tmp,
                &ValidParamCheck::default().add_rule(ValidEmail::default()),
            );
        }
        param_valid
            .add(
                valid_key!("app_id"),
                &app_id,
                &ValidParamCheck::default().add_rule(ValidNumber::id()),
            )
            .add(
                valid_key!("tpl_id"),
                &tpl_id,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .add(
                valid_key!("tpl_var"),
                &tpl_var,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(1, 20000)),
            )
            .add(
                valid_key!("max_try_num"),
                &max_try_num,
                &ValidParamCheck::default().add_rule(ValidNumber::range(0, 5)),
            );
        param_valid.check()?;
        Ok(())
    }
    //添加短信任务
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn add<'t>(
        &self,
        mail: &[&'t str],
        app_id: u64,
        tpl_id: &str,
        tpl_var: &str,
        expected_time: u64,
        reply_mail: Option<&str>,
        user_id: Option<u64>,
        max_try_num: u8,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(u64, Vec<(u64, &'t str)>)> //(body id,<msg id,mail>)
    {
        self.add_param_valid(mail, app_id, tpl_id, tpl_var, reply_mail, max_try_num)
            .await?;
        let user_id = user_id.unwrap_or_default();
        let add_time = now_time().unwrap_or_default();
        let reply_mail = reply_mail.unwrap_or_default().to_string();
        let tpl_id = tpl_id.to_owned();
        let tpl_var = tpl_var.to_owned();
        let mut max_try_num = max_try_num.to_owned();

        let mut idata = Vec::with_capacity(mail.len());
        let reqid = env_data
            .map(|t| t.request_id.to_owned().unwrap_or_default())
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

        let reply_host = if max_try_num == 0 {
            hostname::get()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        } else {
            "".to_string()
        };
        if max_try_num == 0 {
            max_try_num = 1
        }
        if max_try_num > 10 {
            max_try_num = 10
        }

        let body_id = Insert::<SenderMailBodyModel, _>::new(
            lsys_core::model_option_set!(SenderMailBodyModelRef,{
                request_id:reqid,
                app_id:app_id,
                tpl_id:tpl_id,
                tpl_var:tpl_var,
                status:SenderMailBodyStatus::Init as i8,
                add_time:add_time,
                reply_mail:reply_mail,
                max_try_num:max_try_num as u16,
                user_id:user_id,
                user_ip:user_ip,
                expected_time:expected_time,
                reply_host:reply_host,
            }),
        )
        .execute(&mut *tran)
        .await?
        .last_insert_id();
        let res_data = "".to_string();
        for (aid, _, to) in add_data.iter() {
            idata.push(lsys_core::model_option_set!(SenderMailMessageModelRef,{
                snid:aid,
                sender_body_id:body_id,
                to_mail:to,
                try_num:0,
                status:SenderMailMessageStatus::Init as i8,
                send_time:0,
                add_time:add_time,
                res_data:res_data,
            }));
        }

        let tmp = Insert::<SenderMailMessageModel, _>::new_vec(idata)
            .execute(&mut *tran)
            .await;
        if let Err(err) = tmp {
            tran.rollback().await?;
            return Err(err.into());
        }

        tran.commit().await?;

        self.logger
            .add(
                &LogMessage {
                    action: "add",
                    body_id,
                    user_id,
                    message_id: None,
                    sender_type: SenderType::Mailer as i8,
                },
                Some(body_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok((
            body_id,
            add_data.into_iter().map(|e| (e.0, e.1)).collect::<Vec<_>>(),
        ))
    }
    pub(crate) async fn cancel_form_message(
        &self,
        body: &SenderMailBodyModel,
        message: &SenderMailMessageModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<()> {
        if SenderMailMessageStatus::IsCancel.eq(message.status) {
            return Ok(());
        }
        if SenderMailMessageStatus::Init.eq(message.status) {
            let change = lsys_core::model_option_set!(SenderMailMessageModelRef, {
                status: SenderMailMessageStatus::IsCancel as i8
            });
            Update::<SenderMailMessageModel, _>::new(change)
                .execute_by_pk(message, &self.db)
                .await?;

            self.logger
                .add(
                    &LogMessage {
                        action: "cancel",
                        body_id: body.id,
                        message_id: Some(message.id),
                        sender_type: SenderType::Mailer as i8,
                        user_id,
                    },
                    Some(message.id),
                    Some(user_id),
                    None,
                    env_data,
                )
                .await;

            return Ok(());
        }
        Err(SenderError::System(
            fluent_message!("mail-cancel-status-error",{
                    "status":message.status
                }
            ),
        )) //"can't be cancel,status:{}",
    }
    pub async fn find_config_by_id(&self, id: u64) -> SenderResult<SenderConfigModel> {
        self.config.find_by_id(id).await
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn config_add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: SenderMailConfigType,
        config_data: &Value,
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
                                None => {
                                    return Err(SenderError::System(fluent_message!(
                                        "mail-config-add-error",
                                        {
                                            "name":$name,
                                            "msg": $miss_err
                                           }
                                    )))
                                }
                            },
                            None => {
                                return Err(SenderError::System(fluent_message!(
                                    "mail-config-add-error",

                                   {
                                    "name":$name,
                                    "msg": $miss_err
                                   }
                                )));
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
                        return Err(SenderError::System(fluent_message!(
                            "mail-config-add-error",
                            {
                                "name":"range_time,max_send",
                                "msg": err
                             }

                        )));
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
                    return Err(SenderError::System(fluent_message!(
                        "mail-config-add-max-num-error" //"send max need number".to_string()
                    )));
                }
            },
        };
        self.config
            .add(
                app_id,
                priority,
                config_type as i8,
                &config_data,
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
    pub(crate) async fn send_check(
        &self,
        app_id: Option<u64>,
        tpl_id: &str,
        mails: &[&str],
        send_time: u64,
    ) -> SenderResult<()> {
        if mails.is_empty() {
            return Err(SenderError::System(fluent_message!(
                "mail-send-check-miss-error" //"miss to mail box".to_string()
            )));
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
            return Err(SenderError::System(
                fluent_message!("mail-send-check-max-send", //"send mail limit :{}",
                    {
                    "max":max_send
                    }
                ),
            ));
        }
        for (c, r) in rule.iter() {
            match r {
                SenderMailConfigData::Limit(limit) => {
                    if limit.range_time == 0 || limit.max_send == 0 || nowt < limit.range_time {
                        continue;
                    }
                    let msql = mails
                        .iter()
                        .map(|e| {
                            sql_format!(
                                "to_mail={}",
                                string_clear(
                                    e,
                                    StringClear::Option(STRING_CLEAR_FORMAT),
                                    Some(255)
                                )
                            )
                        })
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
                        return Err(SenderError::System(
                            fluent_message!("mail-send-check-block", //"send block on :{} [{}]",
                                {
                                "to":to,
                                "config_id":c.id
                                }
                            ),
                        ));
                    }
                }
                SenderMailConfigData::BlockDomain { domain } => {
                    if mails
                        .iter()
                        .any(|a| a.split('@').nth(1).unwrap_or_default() == *domain)
                    {
                        return Err(SenderError::System(
                            fluent_message!("mail-send-check-block-domain", //"send block on :{} [{}]",
                                {
                                "domain":domain,
                                "config_id":c.id
                                }
                            ),
                        ));
                    }
                }
                SenderMailConfigData::Close => {
                    return Err(SenderError::System(
                        fluent_message!("mail-send-check-close", //"send mail is close"
                            {
                            "config_id":c.id
                            }
                        ),
                    ));
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
                        return Err(SenderError::System(
                            fluent_message!("mail-send-check-limit", //  "trigger limit rule :{} on {} [{}]",
                                {
                                    "max_send":limit.max_send,
                                    "to_mail":&t.2,
                                    "config_id":id
                                }
                            ),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}
