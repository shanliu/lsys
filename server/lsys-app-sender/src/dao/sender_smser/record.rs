use std::{collections::HashSet, sync::Arc};

use crate::{
    dao::{
        MessageLogs, MessageReader, SenderConfig, SenderError, SenderResult, logger::LogMessage,
    },
    model::{
        SenderConfigModel, SenderLogModel, SenderSmsBodyModel, SenderSmsBodyStatus,
        SenderSmsConfigData, SenderSmsConfigLimit, SenderSmsConfigType, SenderSmsMessageModel,
        SenderSmsMessageStatus, SenderType,
    },
};
use lsys_core::db::{CursorPageData, CursorPageParam, OffsetPageParam, TotalParam, TotalRow};
use lsys_core::fluent_message;
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use lsys_core::valid_key;
use lsys_core::valid_param::{
    ValidMobile, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};

use lsys_core::db::{BatchInsert, Insert, QueryBuilderExt, TableMeta, Update, WhereClause};
use lsys_logger::dao::ChangeLoggerDao;
use serde_json::Value;
use sqlx::{MySql, Pool, QueryBuilder};

//短信记录

pub struct SmsRecord {
    db: Pool<sqlx::MySql>,
    config: Arc<SenderConfig>,
    message_logs: Arc<MessageLogs>,
    logger: Arc<ChangeLoggerDao>,
    message_reader: Arc<MessageReader<SenderSmsBodyModel, SenderSmsMessageModel>>,
}

impl SmsRecord {
    #[allow(clippy::too_many_arguments)]
    fn build_message_where(
        wb: &mut WhereClause<'_, '_, MySql>,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<i8>,
        mobile: Option<&str>,
    ) -> bool {
        if let Some(s) = mobile {
            let s = string_clear(s, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
            if s.is_empty() {
                return false;
            }
            wb.and().field_eq("m.mobile", s);
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
        message_reader: Arc<MessageReader<SenderSmsBodyModel, SenderSmsMessageModel>>,
    ) -> Self {
        Self {
            config,
            logger,
            message_logs,
            message_reader,
            db,
        }
    }
    //读取短信任务数据
    //根据ID获取消息
    pub async fn find_message_by_id(&self, id: u64) -> SenderResult<SenderSmsMessageModel> {
        self.message_reader.find_message_by_id(&id).await
    }
    pub async fn find_body_by_id(&self, id: u64) -> SenderResult<SenderSmsBodyModel> {
        self.message_reader.find_body_by_id(&id).await
    }
    //消息数量 (性能优化: 不执行 COUNT(*), 而是查询 threshold+1 条记录来判断)
    #[allow(clippy::too_many_arguments)]
    pub async fn message_count(
        &self,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<SenderSmsMessageStatus>,
        mobile: Option<&str>,
        total_param: &TotalParam,
    ) -> SenderResult<TotalRow> {
        let query = total_param.total_count_query();

        let mut qb = QueryBuilder::<MySql>::new(if query.is_threshold_mode() {
            format!(
                "select count(*) as total from (select 1 from {} as m join {} as b on m.sender_body_id=b.id",
                SenderSmsMessageModel::table_name(),
                SenderSmsBodyModel::table_name(),
            )
        } else {
            format!(
                "select count(*) as total from {} as m join {} as b on m.sender_body_id=b.id",
                SenderSmsMessageModel::table_name(),
                SenderSmsBodyModel::table_name(),
            )
        });
        let mut wb = WhereClause::new(&mut qb);
        if !Self::build_message_where(
            &mut wb,
            user_id,
            app_id,
            tpl_key,
            body_id,
            msg_snid,
            status.map(|s| s as i8),
            mobile,
        ) {
            return Ok(TotalRow::Exact(0));
        }

        if query.is_threshold_mode() {
            query.push_limit(&mut qb);
            qb.push(") as t");
        }

        let count = qb.build_query_scalar::<i64>().fetch_one(&self.db).await?;

        Ok(query.finalize(count))
    }
    //消息列表
    #[allow(clippy::too_many_arguments)]
    pub async fn message_list(
        &self,
        user_id: Option<u64>,
        app_id: Option<u64>,
        tpl_key: Option<&str>,
        body_id: Option<u64>,
        msg_snid: Option<u64>,
        status: Option<SenderSmsMessageStatus>,
        mobile: Option<&str>,
        limit: &CursorPageParam<u64>,
    ) -> SenderResult<(
        Vec<(SenderSmsMessageModel, Option<SenderSmsBodyModel>)>,
        CursorPageData<u64>,
    )> {
        let query_limit = limit.page_query("m.id");
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select m.* from {} as m join {} as b on m.sender_body_id=b.id",
            SenderSmsMessageModel::table_name(),
            SenderSmsBodyModel::table_name(),
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
            mobile,
        ) {
            return Ok((vec![], CursorPageData::default()));
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);

        let mut m_data = qb
            .build_query_as::<SenderSmsMessageModel>()
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
                SenderSmsBodyModel::table_name()
            ));
            pks_qb.push_where().field_in_copied("id", &pks);
            pks_qb
                .build_query_as::<SenderSmsBodyModel>()
                .fetch_all(&self.db)
                .await?
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
    //消息日志数量
    pub async fn message_log_count(&self, message_id: u64) -> SenderResult<i64> {
        self.message_logs.list_count(message_id).await
    }
    //消息日志列表
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
        mobiles: &[(&str, &str)],
        app_id: u64,
        tpl_key: &str,
        tpl_var: &str,
        max_try_num: u8,
    ) -> SenderResult<()> {
        let mut param_valid = ValidParam::default();
        for mt in mobiles {
            param_valid.add(
                valid_key!("mobile"),
                &format!("{}{}", mt.0, mt.1),
                &ValidParamCheck::default().add_rule(ValidMobile::default()),
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
        mobiles: &[(&'t str, &'t str)],
        app_id: u64,
        tpl_key: &str,
        tpl_var: &str,
        expected_time: u64,
        user_id: Option<u64>,
        max_try_num: u8,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<(u64, Vec<(u64, &'t str, &'t str)>)> //返回(body id,<msg id,area,mobile>)
    {
        self.add_param_valid(mobiles, app_id, tpl_key, tpl_var, max_try_num)
            .await?;
        let user_id = user_id.unwrap_or_default();
        let add_time = now_time().unwrap_or_default();
        let tpl_key = tpl_key.to_owned();
        let tpl_var = tpl_var.to_owned();
        let mut max_try_num = max_try_num.to_owned();
        let add_data = mobiles
            .iter()
            .map(|e| {
                let id = self.message_reader.message_id();

                (id, e.0, e.1, e.0.to_owned(), e.1.to_owned())
            })
            .collect::<Vec<_>>();

        let mut tran = self.db.begin().await?;
        let user_ip = env_data
            .map(|e| e.request_ip.clone().unwrap_or_default())
            .unwrap_or_default();
        let reqid = env_data
            .map(|t| t.request_id.to_owned().unwrap_or_default())
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
        let res = Insert::<_, SenderSmsBodyModel>::new()
            .set(SenderSmsBodyModel::APP_ID, app_id)
            .set(SenderSmsBodyModel::TPL_KEY, &tpl_key)
            .set(SenderSmsBodyModel::REQUEST_ID, reqid)
            .set(SenderSmsBodyModel::TPL_VAR, &tpl_var)
            .set(SenderSmsBodyModel::REPLY_HOST, reply_host)
            .set(SenderSmsBodyModel::STATUS, SenderSmsBodyStatus::Init as i8)
            .set(SenderSmsBodyModel::ADD_TIME, add_time)
            .set(SenderSmsBodyModel::MAX_TRY_NUM, max_try_num as u16)
            .set(SenderSmsBodyModel::USER_ID, user_id)
            .set(SenderSmsBodyModel::USER_IP, user_ip)
            .set(SenderSmsBodyModel::EXPECTED_TIME, expected_time)
            .execute(&mut *tran)
            .await;
        let body_id = match res {
            Ok(e) => e.last_insert_id(),
            Err(err) => {
                tran.rollback().await?;
                return Err(err.into());
            }
        };
        let res_data = "";
        let mut batch = BatchInsert::<_, SenderSmsMessageModel>::with_capacity(add_data.len());
        for (id, _, _, area, mobile) in add_data.iter() {
            batch = batch.push(
                Insert::<_, SenderSmsMessageModel>::new()
                    .set(SenderSmsMessageModel::SNID, *id)
                    .set(SenderSmsMessageModel::SENDER_BODY_ID, body_id)
                    .set(SenderSmsMessageModel::MOBILE, mobile)
                    .set(SenderSmsMessageModel::AREA, area)
                    .set(SenderSmsMessageModel::TRY_NUM, 0u16)
                    .set(
                        SenderSmsMessageModel::STATUS,
                        SenderSmsMessageStatus::Init as i8,
                    )
                    .set(SenderSmsMessageModel::SEND_TIME, 0u64)
                    .set(SenderSmsMessageModel::ADD_TIME, add_time)
                    .set(SenderSmsMessageModel::RES_DATA, res_data),
            );
        }
        batch.execute(&mut *tran).await?;

        tran.commit().await?;

        self.logger
            .add(
                &LogMessage {
                    action: "add",
                    sender_type: SenderType::Smser as i8,
                    body_id,
                    message_id: None,
                    user_id,
                },
                Some(body_id),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok((
            body_id,
            add_data
                .into_iter()
                .map(|e| (e.0, e.1, e.2))
                .collect::<Vec<_>>(),
        ))
    }
    //取消短信发送
    pub(crate) async fn cancel_form_message(
        &self,
        body: &SenderSmsBodyModel,
        message: &SenderSmsMessageModel,
        user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> SenderResult<()> {
        if SenderSmsMessageStatus::IsCancel.eq(message.status) {
            return Ok(());
        }
        if SenderSmsMessageStatus::Init.eq(message.status) {
            Update::<_, SenderSmsMessageModel>::new()
                .set(
                    SenderSmsMessageModel::STATUS,
                    SenderSmsMessageStatus::IsCancel as i8,
                )
                .execute(&self.db, |qb| {
                    qb.push_where().field_eq("id", message.id);
                })
                .await?;

            self.logger
                .add(
                    &LogMessage {
                        action: "cancel",
                        sender_type: SenderType::Smser as i8,
                        body_id: body.id,
                        user_id,
                        message_id: Some(message.id),
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
            fluent_message!("sms-cancel-status-error",{
                    "status":message.status
                }
            ),
        )) //"can't be cancel,status:{}",
        // Err(SenderError::System(

        //     format!(
        //     "can't be cancel,status:{}",
        //     message.status
        // )))
    }
    //查找短信基本配置
    pub async fn find_config_by_id(&self, id: u64) -> SenderResult<SenderConfigModel> {
        self.config.find_by_id(id).await
    }
    //短信配置添加
    #[allow(clippy::too_many_arguments)]
    pub async fn config_add(
        &self,
        app_id: Option<u64>,
        priority: i8,
        config_type: SenderSmsConfigType,
        config_data: &Value,
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
                                None => {
                                    return Err(SenderError::System(fluent_message!(
                                        "sms-config-add-error",
                                        {
                                            "name":$name,
                                            "msg": $miss_err
                                           }
                                    )))
                                }
                                // None => return Err(SenderError::System($wrong_err.to_string())),
                            },
                            None => {
                                return Err(SenderError::System(fluent_message!(
                                    "sms-config-add-error",
                                    {
                                        "name":$name,
                                        "msg": $miss_err
                                       }
                                )));
                            } // None => {

                              //     return Err(SenderError::System($miss_err.to_string()));
                              // }
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
                        return Err(SenderError::System(fluent_message!(
                            "sms-config-add-error",
                            {
                                "name":"range_time,max_send",
                                "msg": err
                             }
                        )));
                        // return Err(SenderError::System(err.to_string()));
                    }
                }
            }
            SenderSmsConfigType::Block => config_data.to_string(),
            SenderSmsConfigType::Close => "".to_string(),
            SenderSmsConfigType::PassTpl => config_data.to_string(),
            SenderSmsConfigType::MaxOfSend => match config_data.as_u64() {
                Some(num) => (num as u32).to_string(),
                None => {
                    return Err(SenderError::System(fluent_message!(
                        "sms-config-add-max-num-error" //"send max need number".to_string()
                    )));
                    // return Err(SenderError::System("send max need number".to_string()));
                }
            },
        };
        let id = self
            .config
            .add(
                app_id,
                priority,
                config_type as i8,
                &config_data,
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
    pub(crate) async fn send_check(
        &self,
        app_id: Option<u64>,
        tpl_key: &str,
        mobiles: &[(&str, &str)],
        send_time: u64,
    ) -> SenderResult<()> {
        if mobiles.is_empty() {
            return Err(SenderError::System(fluent_message!(
                "sms-send-check-miss-error" //"miss to sms box".to_string()
            )));
            // return Err(SenderError::System("miss mobile".to_string()));
        }

        let mut rule = self
            .config_list(None, None, Some(app_id.unwrap_or_default()))
            .await?;
        let nowt = send_time;
        rule.sort_by(|a, b| a.0.priority.cmp(&b.0.priority));
        if let Some(max_send) = (|| {
            for t in rule.iter() {
                if let SenderSmsConfigData::MaxOfSend(u) = t.1 {
                    return Some(u);
                }
            }
            None
        })() && mobiles.len() > max_send as usize
        {
            return Err(SenderError::System(
                fluent_message!("sms-send-check-max-send", //"send sms limit :{}",
                    {
                    "max":max_send
                    }
                ),
            ));
        }
        let mut limit_config: Vec<(u64, &SenderSmsConfigLimit)> = vec![];
        let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM (");
        for (c, r) in rule.iter() {
            match r {
                SenderSmsConfigData::Limit(limit) => {
                    if limit.range_time == 0 || limit.max_send == 0 || nowt < limit.range_time {
                        continue;
                    }
                    let stime = nowt - limit.range_time;
                    if !limit_config.is_empty() {
                        qb.push(" UNION ALL ");
                    }
                    qb.push("select count(*) as total,");
                    qb.push(c.id);
                    qb.push(format!(
                        " as limit_id,m.area,m.mobile from {} as b join {} as m \
                        on m.sender_body_id=b.id",
                        SenderSmsBodyModel::table_name(),
                        SenderSmsMessageModel::table_name(),
                    ));
                    qb.push_where().field_eq("b.app_id", c.app_id);
                    qb.push_and().field_in_copied(
                        "m.status",
                        &[
                            SenderSmsMessageStatus::IsSend as i8,
                            SenderSmsMessageStatus::IsReceived as i8,
                        ],
                    );
                    qb.push_and().field_gte("b.expected_time", stime);
                    qb.push_and().push("(");
                    for (i, e) in mobiles.iter().enumerate() {
                        if i > 0 {
                            qb.push(" or ");
                        }
                        qb.push("(");
                        qb.field_eq(
                            "area",
                            string_clear(e.0, StringClear::Option(STRING_CLEAR_FORMAT), Some(12)),
                        );
                        qb.push_and().field_eq(
                            "mobile",
                            string_clear(e.1, StringClear::Option(STRING_CLEAR_FORMAT), Some(33)),
                        );
                        qb.push(")");
                    }
                    qb.push(") group by m.area,m.mobile");
                    limit_config.push((c.id, limit));
                }
                SenderSmsConfigData::PassTpl(itpl_key) => {
                    if *tpl_key == *itpl_key {
                        break;
                    }
                }
                SenderSmsConfigData::Block { area, mobile } => {
                    if mobiles.iter().any(|a| *a.0 == *area && *a.1 == *mobile) {
                        return Err(SenderError::System(
                            fluent_message!("sms-send-check-block", //"send block on :{} [{}]",
                                {
                                    "area":area,
                                "mobile":mobile,
                                "config_id":c.id
                                }
                            ),
                        ));
                        // return Err(SenderError::System(format!(
                        //     "send block on :{}{} [{}]",
                        //     area, mobile, c.id
                        // )));
                    }
                }
                SenderSmsConfigData::Close => {
                    return Err(SenderError::System(
                        fluent_message!("sms-send-check-close", //"send sms is close"
                            {
                            "config_id":c.id
                            }
                        ),
                    ));
                    // return Err(SenderError::System("send sms is close".to_string()));
                }
                _ => {}
            }
        }
        if !limit_config.is_empty() {
            qb.push(") AS t");
            let data = qb
                .build_query_as::<(i64, i64, String, String)>()
                .fetch_all(&self.db)
                .await?;
            for (id, limit) in limit_config {
                if let Some(t) = data.iter().find(|e| e.1 as u64 == id)
                    && t.0 >= limit.max_send.into()
                {
                    return Err(SenderError::System(
                        fluent_message!("sms-send-check-limit", //  "trigger limit rule :{} on {} [{}]",
                            {
                                "max_send":limit.max_send,
                                "area":&t.2,
                                "mobile":&t.3,
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
