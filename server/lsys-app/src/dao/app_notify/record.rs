use std::sync::Arc;
use std::time::Duration;

use crate::dao::logger::{AppNotifyConfigLog, AppNotifyDataDelLog};
use crate::dao::{AppError, AppResult};
use crate::model::{
    AppModel, AppNotifyConfigModel, AppNotifyConfigModelRef, AppNotifyDataModel,
    AppNotifyDataModelRef, AppNotifyDataStatus, AppNotifyTryTimeMode, AppNotifyType,
};
use lsys_core::{
    fluent_message, now_time, string_clear, valid_key, LimitParam, RequestEnv, ValidNumber,
    ValidParam, ValidParamCheck, ValidPattern, ValidStrlen, ValidUrl,
};

use lsys_core::db::{Insert, ModelTableName, SqlExpr, Update, WhereOption};
use lsys_core::{model_option_set, sql_format};
use lsys_logger::dao::ChangeLoggerDao;
use reqwest::Method;
use sqlx::{FromRow, Pool, Row};

use lsys_core::db::SqlQuote;
use tracing::warn;

pub struct AppNotifyRecord {
    db: Pool<sqlx::MySql>,
    logger: Arc<ChangeLoggerDao>,
}

impl AppNotifyRecord {
    pub fn new(db: Pool<sqlx::MySql>, logger: Arc<ChangeLoggerDao>) -> Self {
        Self { db, logger }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_data_by_id,
        u64,
        AppNotifyDataModel,
        AppResult<AppNotifyDataModel>,
        id,
        "id={id}"
    );
    pub async fn find_config_by_app(
        &self,
        app_id: u64,
        notify_method: &str,
    ) -> AppResult<AppNotifyConfigModel> {
        let method = string_clear(notify_method, lsys_core::StringClear::Ident, Some(65));
        let data = sqlx::query_as::<_, AppNotifyConfigModel>(&sql_format!(
            "select * from {} where app_id={} and notify_method={}",
            AppNotifyConfigModel::table_name(),
            app_id,
            method
        ))
        .fetch_one(&self.db)
        .await?;
        Ok(data)
    }
    pub async fn find_config_by_apps(
        &self,
        app_id: &[u64],
        notify_method: &str,
    ) -> AppResult<Vec<AppNotifyConfigModel>> {
        if app_id.is_empty() {
            return Ok(vec![]);
        }
        let method = string_clear(notify_method, lsys_core::StringClear::Ident, Some(65));
        let data = sqlx::query_as::<_, AppNotifyConfigModel>(&sql_format!(
            "select * from {} where app_id in ({}) and notify_method={}",
            AppNotifyConfigModel::table_name(),
            app_id,
            method
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(data)
    }
    async fn set_app_config_param_valid(
        &self,
        notify_method: &str,
        call_url: &str,
    ) -> AppResult<()> {
        ValidParam::default()
            .add(
                valid_key!("notify_method"),
                &notify_method,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, 64))
                    .add_rule(ValidPattern::Ident),
            )
            .add(
                valid_key!("call_url"),
                &call_url,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, 512))
                    .add_rule(ValidUrl::default()),
            )
            .check()?;
        Ok(())
    }
    pub async fn set_app_config(
        &self,
        app: &AppModel,
        notify_method: &str,
        call_url: &str,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<u64> {
        self.set_app_config_param_valid(notify_method, call_url)
            .await?;
        let client = reqwest::Client::builder();
        let client = client
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| AppError::System(fluent_message!("notify-reqwest-build-error", e)))?;
        client
            .request(Method::POST, call_url)
            .send()
            .await
            .map_err(|e| {
                AppError::System(fluent_message!("notify-reqwest-check-error", {
                    "msg":e,
                    "url":call_url,
                }))
            })?;

        let call_url = call_url.to_owned();
        let change_user_id = change_user_id.to_owned();
        let create_time = now_time().unwrap_or_default();
        let id = match self.find_config_by_app(app.id, notify_method).await {
            Ok(row) => {
                let change = lsys_core::model_option_set!(AppNotifyConfigModelRef,{
                    call_url:call_url,
                    change_time:create_time,
                    change_user_id:change_user_id,
                });
                Update::<AppNotifyConfigModel, _>::new(change)
                    .execute_by_pk(&row, &self.db)
                    .await?;
                row.id
            }
            Err(AppError::Sqlx(sqlx::Error::RowNotFound)) => {
                let notify_method = notify_method.to_owned();
                let res = Insert::<AppNotifyConfigModel, _>::new(
                    model_option_set!(AppNotifyConfigModelRef ,{
                        app_id: app.id,
                        notify_method: notify_method,
                        call_url:call_url,
                        app_user_id:app.user_id,
                        change_user_id: change_user_id,
                        create_time: create_time,
                    }),
                )
                .execute(&self.db)
                .await
                .map_err(|e| {
                    warn!("add notify error fail:{}", e);
                    e
                })?;
                res.last_insert_id()
            }
            Err(err) => {
                return Err(err);
            }
        };

        self.logger
            .add(
                &AppNotifyConfigLog {
                    notify_method,
                    url: &call_url,
                    user_id: change_user_id,
                },
                Some(id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(id)
    }
    async fn add_param_valid(
        &self,
        notify_method: &str,
        notify_key: &str,
        notify_data: &str,
        try_max: u8,
        try_delay: u16,
    ) -> AppResult<()> {
        ValidParam::default()
            .add(
                valid_key!("notify_method"),
                &notify_method,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, 64))
                    .add_rule(ValidPattern::Ident),
            )
            .add(
                valid_key!("notify_key"),
                &notify_key,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(0, 64))
                    .add_rule(ValidPattern::Ident),
            )
            .add(
                valid_key!("try_max"),
                &try_max,
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, 30)),
            )
            .add(
                valid_key!("try_delay"),
                &try_delay,
                &ValidParamCheck::default().add_rule(ValidNumber::range(1, 3600)),
            )
            .add(
                valid_key!("notify_data"),
                &notify_data,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(0, 20000)),
            )
            .check()?;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) async fn add(
        &self,
        app_id: u64,
        notify_method: &str,
        notify_type: AppNotifyType,
        notify_key: &str,
        notify_data: &str,
        try_max: u8,
        try_mode: AppNotifyTryTimeMode,
        try_delay: u16,
        clear_init_status: bool,
    ) -> AppResult<u64> {
        self.add_param_valid(notify_method, notify_key, notify_data, try_max, try_delay)
            .await?;
        let notify_type = notify_type as u8;
        let notify_method = notify_method.to_owned();
        let notify_key = notify_key.to_owned();
        let notify_data = notify_data.to_owned();
        let create_time = now_time().unwrap_or_default();
        let try_mode = try_mode as i8;
        let status = AppNotifyDataStatus::Init as i8;

        let next_time = if clear_init_status {
            0
        } else {
            match sqlx::query_scalar::<_, Option<u64>>(&sql_format!(
                "select max(next_time) from {} 
                            where app_id={} and notify_method={} and notify_key={} and status={}",
                AppNotifyDataModel::table_name(),
                &app_id,
                &notify_method,
                &notify_key,
                status
            ))
            .fetch_one(&self.db)
            .await
            {
                Ok(t) => t.unwrap_or_default(),
                Err(sqlx::Error::RowNotFound) => 0,
                Err(err) => Err(err)?,
            }
        };
        let mut remove_history_count = 0;
        let mut tdb = self.db.begin().await?;
        let res = Insert::<AppNotifyDataModel, _>::new(model_option_set!(AppNotifyDataModelRef ,{
            app_id:app_id,
            notify_method: notify_method,
            notify_payload: notify_data,
            notify_type:notify_type,
            notify_key:notify_key,
            status: status,
            try_max:try_max,
            try_mode:try_mode ,
            next_time:next_time,
            try_delay:try_delay ,
            create_time: create_time,
        }))
        .execute(&mut *tdb)
        .await
        .map_err(|e| {
            warn!("add notify error fail:{}", e);
            e
        });
        let last_id = match res {
            Ok(t) => t.last_insert_id(),
            Err(err) => {
                tdb.rollback().await?;
                return Err(err)?;
            }
        };
        if clear_init_status {
            let del_status = AppNotifyDataStatus::Delete as i8;
            let change = model_option_set!(AppNotifyDataModelRef,{
                status:del_status,
                delete_time:create_time
            });
            match Update::<AppNotifyDataModel, _>::new(change)
                .execute_by_where(
                    &WhereOption::Where(sql_format!(
                        "app_id={} and notify_method={} and notify_key={} and id<{} and status={}",
                        &app_id,
                        &notify_method,
                        &notify_key,
                        last_id,
                        status
                    )),
                    &mut *tdb,
                )
                .await
            {
                Ok(c) => remove_history_count = c.rows_affected(),
                Err(err) => {
                    tdb.rollback().await?;
                    return Err(err.into());
                }
            };
        }
        tdb.commit().await?;

        if remove_history_count > 0 {
            self.logger
                .add(
                    &AppNotifyDataDelLog {
                        source: "add",
                        info: &format!("del num:{},trigger id:{}", remove_history_count, last_id),
                    },
                    None,
                    Some(0),
                    None,
                    None,
                )
                .await;
        }

        Ok(last_id)
    }
    //删除回调
    pub(crate) async fn del(
        &self,
        notify_id: u64,
        del_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        let data = self.find_data_by_id(&notify_id).await?;
        if AppNotifyDataStatus::Delete.eq(data.status) {
            return Ok(());
        }
        let create_time = now_time().unwrap_or_default();
        let del_status = AppNotifyDataStatus::Delete as i8;
        let change = model_option_set!(AppNotifyDataModelRef,{
            status:del_status,
            delete_time:create_time
        });
        Update::<AppNotifyDataModel, _>::new(change)
            .execute_by_where(
                &WhereOption::Where(sql_format!("id={}", notify_id)),
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &AppNotifyDataDelLog {
                    source: "del",
                    info: "",
                },
                Some(notify_id),
                Some(del_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    fn data_sql(
        &self,
        app_id: Option<u64>,
        app_user_id: Option<u64>,
        notify_method: Option<&str>,
        notify_key: Option<&str>,
        status: Option<AppNotifyDataStatus>,
    ) -> Option<Vec<String>> {
        let mut sqlwhere = vec![];
        if let Some(s) = notify_method {
            let s = string_clear(s, lsys_core::StringClear::Ident, Some(65));
            if s.is_empty() {
                return None;
            }
            sqlwhere.push(sql_format!("d.notify_method={}", s));
        }
        if let Some(s) = notify_key {
            let s = string_clear(s, lsys_core::StringClear::Ident, Some(65));
            sqlwhere.push(sql_format!("d.notify_key={}", s));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("d.app_id = {}  ", aid));
        }
        if let Some(uid) = app_user_id {
            sqlwhere.push(sql_format!(
                "d.app_id in ( select app_id from {} where app_user_id = {} )",
                AppNotifyConfigModel::table_name(),
                uid
            ));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("d.status={} ", s));
        }
        Some(sqlwhere)
    }

    //消息数量
    pub async fn data_count(
        &self,
        app_id: Option<u64>,
        app_user_id: Option<u64>,
        notify_method: Option<&str>,
        notify_key: Option<&str>,
        status: Option<AppNotifyDataStatus>,
    ) -> AppResult<i64> {
        let sqlwhere = match self.data_sql(app_id, app_user_id, notify_method, notify_key, status) {
            Some(s) => s,
            None => return Ok(0),
        };

        let sql = sql_format!(
            "select count(*) as total from {} as d {}",
            AppNotifyDataModel::table_name(),
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
    //消息列表
    pub async fn data_list(
        &self,
        app_id: Option<u64>,
        app_user_id: Option<u64>,
        notify_method: Option<&str>,
        notify_key: Option<&str>,
        status: Option<AppNotifyDataStatus>,
        limit: Option<&LimitParam>,
    ) -> AppResult<(Vec<(AppNotifyDataModel, String)>, Option<u64>)> {
        let sqlwhere = match self.data_sql(app_id, app_user_id, notify_method, notify_key, status) {
            Some(s) => s,
            None => return Ok((vec![], None)),
        };
        let where_sql = if let Some(page) = limit {
            let page_where = page.where_sql(
                "d.id",
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
                page.order_sql("d.id"),
                page.limit_sql(),
            )
        } else {
            format!(
                "{} {}  order by d.id desc",
                if !sqlwhere.is_empty() { "where " } else { "" },
                sqlwhere.join(" and ")
            )
        };

        let sql = sql_format!(
            "select d.*,c.call_url from {} as d left join {} as c on d.app_id=c.app_id and d.notify_method=c.notify_method
            {}",
            AppNotifyDataModel::table_name(),
            AppNotifyConfigModel::table_name(),
            SqlExpr(where_sql)
        );

        let mut m_data = sqlx::query(sql.as_str())
            .try_map(
                |row: sqlx::mysql::MySqlRow| match AppNotifyDataModel::from_row(&row) {
                    Ok(res) => {
                        let call_url = row.try_get::<String, &str>("call_url").unwrap_or_default();
                        Ok((res, call_url))
                    }
                    Err(err) => Err(err),
                },
            )
            .fetch_all(&self.db)
            .await?;

        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut m_data))
            .unwrap_or_default();

        Ok((m_data, next.map(|t| t.0.id)))
    }
}
