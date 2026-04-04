use std::{collections::HashSet, sync::Arc};

use crate::{
    dao::{
        logger::LogMessage, MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult,
    },
    model::{
        SenderConfigModel, SenderLogModel, SenderMailBodyModel, SenderMailBodyStatus,
        SenderMailConfigData, SenderMailConfigLimit, SenderMailConfigType, SenderMailMessageModel,
        SenderMailMessageStatus, SenderType,
    },
};
use lsys_core::db::{CursorPageData, CursorPageParam, OffsetPageParam, TotalParam, TotalRow};
use lsys_core::fluent_message;
use lsys_core::utils::{now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT};
use lsys_core::valid_key;
use lsys_core::valid_param::{
    ValidEmail, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};

use lsys_core::db::{BatchInsert, Insert, QueryBuilderExt, TableMeta, Update, WhereClause};
use lsys_logger::dao::ChangeLoggerDao;
use serde_json::Value;
use sqlx::{MySql, Pool, QueryBuilder};

//短信任务记录

pub struct MailRecord {
    db: Pool<sqlx::MySql>,
    config: Arc<SenderConfig>,
    logger: Arc<ChangeLoggerDao>,
    message_logs: Arc<MessageLogs>,
    message_reader: Arc<MessageReader<SenderMailBodyModel, SenderMailMessageModel>>,
}

impl MailRecord {
    #[allow(clippy::too_many_arguments)]
    fn build_message_where(
        wb: &mut WhereClause<'_, '_, MySql>,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<i8>,
        to_mail: Option<&str>,
    ) -> bool {
        if let Some(s) = to_mail {
            let s = string_clear(s, StringClear::Option(STRING_CLEAR_FORMAT), Some(255));
            if s.is_empty() {
                return false;
            }
            wb.and().field_eq("m.to_mail", s);
        }
        if let Some(aid) = app_id {
            wb.and().field_eq("b.app_id", aid);
        }
        if let Some(uid) = user_id {
            wb.and().field_eq("b.user_id", uid);
        }
        if let Some(t) = tpl_key {
            let t = string_clear(t, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if t.is_empty() {
                return false;
            }
            wb.and().field_eq("b.tpl_key", t);
        }
        if let Some(s) = status {
            wb.and().field_eq("m.status", s);
        }
        if let Some(s) = body_id {
            wb.and().field_eq("m.sender_body_id", s);
        }
        if let Some(s) = msg_snid {
            wb.and().field_eq("m.snid", s);
        }
        true
    }

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
        tpl_key: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<SenderMailMessageStatus>,
        to_mail: Option<&str>,
        total_param: &TotalParam,
    ) -> SenderResult<TotalRow> {
        let query = total_param.total_count_query();

        let mut qb = QueryBuilder::<MySql>::new(
            if query.is_threshold_mode() {
                format!(
                    "select count(*) as total from (select 1 from {} as m join {} as b on m.sender_body_id=b.id",
                    SenderMailMessageModel::table_name(),
                    SenderMailBodyModel::table_name(),
                )
            } else {
                format!(
                    "select count(*) as total from {} as m join {} as b on m.sender_body_id=b.id",
                    SenderMailMessageModel::table_name(),
                    SenderMailBodyModel::table_name(),
                )
            }
        );
        let mut wb = WhereClause::new(&mut qb);
        if !Self::build_message_where(
            &mut wb,
            user_id,
            app_id,
            tpl_key,
            body_id,
            msg_snid,
            status.map(|s| s as i8),
            to_mail,
        ) {
            return Ok(TotalRow::Exact(0));
        }

        if query.is_threshold_mode() {
            query.push_limit(&mut qb);
            qb.push(") as t");
        }

        let count = qb.build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await? as u64;

        Ok(query.finalize(count))
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn message_list(
        &self,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<SenderMailMessageStatus>,
        to_mail: Option<&str>,
        limit: &CursorPageParam<u64>,
    ) -> SenderResult<(
        Vec<(SenderMailMessageModel, Option<SenderMailBodyModel>)>,
        CursorPageData<u64>,
    )> {
        let query_limit = limit.page_query("m.id");
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select m.* from {} as m join {} as b on m.sender_body_id=b.id",
            SenderMailMessageModel::table_name(),
            SenderMailBodyModel::table_name(),
        ));
        let mut wb = WhereClause::new(&mut qb);
        let has_cursor = query_limit.has_cursor();
        if has_cursor {
            query_limit.push_where(wb.and());
        }
        if !Self::build_message_where(
            &mut wb,
            user_id,
            app_id,
            tpl_key,
            body_id,
            msg_snid,
            status.map(|s| s as i8),
            to_mail,
        ) {
            return Ok((vec![], CursorPageData::default()));
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);

        let mut m_data = qb.build_query_as::<SenderMailMessageModel>()
            .fetch_all(&self.db)
            .await?;

        let next = query_limit.finalize(&mut m_data, |c, d| *d == c.id, |c| c.id);

        let pks = m_data
            .iter()
            .map(|t| t.sender_body_id)
            .collect::<HashSet<u64>>()
            .into_iter()
            .collect::<Vec<u64>>();

        let b_data = if !pks.is_empty() {
            let mut pks_qb: QueryBuilder<'_, MySql> = QueryBuilder::new(format!(
                "select * from {}",
                SenderMailBodyModel::table_name()
            ));
            pks_qb.push_where().field_in_copied("id", &pks);
            pks_qb.build_query_as::<SenderMailBodyModel>()
                .fetch_all(&self.db).await?
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

        Ok((out_data, next))
    }
    pub async fn message_log_count(&self, message_id: u64) -> SenderResult<i64> {
        self.message_logs.list_count(message_id).await
    }
    pub async fn message_log_list(
        &self,
        message_id: u64,
        page: &OffsetPageParam,
    ) -> SenderResult<Vec<SenderLogModel>> {
        self.message_logs.list_data(message_id, page).await
    }
    #[allow(clippy::too_many_arguments)]
    async fn add_param_valid(
        &self,
        mail: &[&str],
        app_id: u64,
        tpl_key: &str,
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
                valid_key!("tpl_key"),
                &tpl_key,
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
        tpl_key: &str,
        tpl_var: &str,
        expected_time: u64,
        reply_mail: Option<&str>,
        user_id: Option<u64>,
        max_try_num: u8,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(u64, Vec<(u64, &'t str)>)> //(body id,<msg id,mail>)
    {
        self.add_param_valid(mail, app_id, tpl_key, tpl_var, reply_mail, max_try_num)
            .await?;
        let user_id = user_id.unwrap_or_default();
        let add_time = now_time().unwrap_or_default();
        let reply_mail = reply_mail.unwrap_or_default().to_string();
        let tpl_key = tpl_key.to_owned();
        let tpl_var = tpl_var.to_owned();
        let mut max_try_num = max_try_num.to_owned();

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

        let body_id = Insert::<_, SenderMailBodyModel>::new()
            .set(SenderMailBodyModel::REQUEST_ID, reqid)
            .set(SenderMailBodyModel::APP_ID, app_id)
            .set(SenderMailBodyModel::TPL_KEY, &tpl_key)
            .set(SenderMailBodyModel::TPL_VAR, &tpl_var)
            .set(
                SenderMailBodyModel::STATUS,
                SenderMailBodyStatus::Init as i8,
            )
            .set(SenderMailBodyModel::ADD_TIME, add_time)
            .set(SenderMailBodyModel::REPLY_MAIL, &reply_mail)
            .set(SenderMailBodyModel::MAX_TRY_NUM, max_try_num as u16)
            .set(SenderMailBodyModel::USER_ID, user_id)
            .set(SenderMailBodyModel::USER_IP, user_ip)
            .set(SenderMailBodyModel::EXPECTED_TIME, expected_time)
            .set(SenderMailBodyModel::REPLY_HOST, reply_host)
            .execute(&mut *tran)
            .await?
            .last_insert_id();
        let res_data = "";
        let mut batch = BatchInsert::<_, SenderMailMessageModel>::with_capacity(add_data.len());
        for (aid, _, to) in add_data.iter() {
            batch = batch.push(
                Insert::<_, SenderMailMessageModel>::new()
                    .set(SenderMailMessageModel::SNID, *aid)
                    .set(SenderMailMessageModel::SENDER_BODY_ID, body_id)
                    .set(SenderMailMessageModel::TO_MAIL, to)
                    .set(SenderMailMessageModel::TRY_NUM, 0u16)
                    .set(
                        SenderMailMessageModel::STATUS,
                        SenderMailMessageStatus::Init as i8,
                    )
                    .set(SenderMailMessageModel::SEND_TIME, 0u64)
                    .set(SenderMailMessageModel::ADD_TIME, add_time)
                    .set(SenderMailMessageModel::RES_DATA, res_data),
            );
        }

        let tmp = batch.execute(&mut *tran).await;
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
            Update::<_, SenderMailMessageModel>::new()
                .set(
                    SenderMailMessageModel::STATUS,
                    SenderMailMessageStatus::IsCancel as i8,
                )
                .execute(&self.db, |qb| {
                    qb.push_where().field_eq("id", message.id);
                })
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
        tpl_key: &str,
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
        let nowt = send_time;
        rule.sort_by(|a, b| a.0.priority.cmp(&b.0.priority));
        if let Some(max_send) = (|| {
            for t in rule.iter() {
                if let SenderMailConfigData::MaxOfSend(u) = t.1 {
                    return Some(u);
                }
            }
            None
        })()
            && mails.len() > max_send as usize {
                return Err(SenderError::System(
                    fluent_message!("mail-send-check-max-send", //"send mail limit :{}",
                        {
                        "max":max_send
                        }
                    ),
                ));
            }
        let mut limit_config: Vec<(u64, &SenderMailConfigLimit)> = vec![];
        let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM (");
        for (c, r) in rule.iter() {
            match r {
                SenderMailConfigData::Limit(limit) => {
                    if limit.range_time == 0 || limit.max_send == 0 || nowt < limit.range_time {
                        continue;
                    }
                    let stime = nowt - limit.range_time;
                    if !limit_config.is_empty() {
                        qb.push(" UNION ALL ");
                    }
                    qb.push("select count(*) as total,");
                    qb.push_bind(c.id);
                    qb.push(format!(
                        " as limit_id,m.to_mail from {} as b join {} as m\
                         on m.sender_body_id=b.id",
                        SenderMailBodyModel::table_name(),
                        SenderMailMessageModel::table_name(),
                    ));
                    qb.push_where().field_eq("b.app_id", c.app_id);
                    qb.push_and().field_in_copied("m.status", &[SenderMailMessageStatus::IsSend as i8, SenderMailMessageStatus::IsReceived as i8]);
                    qb.push_and().field_gte("b.expected_time", stime);
                    qb.push_and().push("(");
                    for (i, e) in mails.iter().enumerate() {
                        if i > 0 { qb.push(" or "); }
                        qb.field_eq("to_mail", 
                            string_clear(e, StringClear::Option(STRING_CLEAR_FORMAT), Some(255))
                        );
                    }
                    qb.push(") group by m.to_mail");
                    limit_config.push((c.id, limit));
                }
                SenderMailConfigData::PassTpl(itpl_key) => {
                    if *tpl_key == *itpl_key {
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
        if !limit_config.is_empty() {
            qb.push(") AS t");
            let data = qb.build_query_as::<(i64, i64, String)>()
                .fetch_all(&self.db)
                .await?;
            for (id, limit) in limit_config {
                if let Some(t) = data.iter().find(|e| e.1 as u64 == id)
                    && t.0 >= limit.max_send.into() {
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
        Ok(())
    }
}
