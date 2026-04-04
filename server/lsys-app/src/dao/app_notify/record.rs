use std::sync::Arc;
use std::time::Duration;

use crate::dao::logger::{AppNotifyConfigLog, AppNotifyDataDelLog};
use crate::dao::{AppError, AppResult};
use crate::model::{
    AppModel, AppNotifyConfigModel, AppNotifyDataModel, AppNotifyDataStatus, AppNotifyTryTimeMode,
    AppNotifyType,
};
use lsys_core::utils::{now_time, string_clear, RequestEnv, StringClear};
use lsys_core::valid_param::{
    ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen, ValidUrl,
};
use lsys_core::{db::utils::FetchField, fluent_message, valid_key};

use lsys_core::db::{CursorPageData, CursorPageParam, Insert, QueryBuilderExt, TableMeta, TotalParam, TotalRow, Update, WhereClause};
use lsys_logger::dao::ChangeLoggerDao;
use reqwest::Method;
use sqlx::{FromRow, MySql, Pool, QueryBuilder, Row};

use tracing::warn;

pub struct AppNotifyRecord {
    db: Pool<sqlx::MySql>,
    logger: Arc<ChangeLoggerDao>,
}

impl AppNotifyRecord {
    pub fn new(db: Pool<sqlx::MySql>, logger: Arc<ChangeLoggerDao>) -> Self {
        Self { db, logger }
    }
    pub async fn find_data_by_id(&self, id: &u64) -> AppResult<AppNotifyDataModel> {
        Ok(lsys_core::db::utils::Fetch::<MySql, AppNotifyDataModel>::one(
            &self.db,
            |qb| {
                qb.field_eq("id", *id);
            },
        )
        .await?)
    }
    pub async fn find_config_by_app(
        &self,
        app_id: u64,
        notify_method: &str,
    ) -> AppResult<AppNotifyConfigModel> {
        let method = string_clear(notify_method, StringClear::Ident, Some(65));
        Ok(lsys_core::db::utils::Fetch::<MySql, AppNotifyConfigModel>::one(
            &self.db,
            move |qb| {
                qb.field_eq("app_id", app_id);
                qb.push_and().field_eq("notify_method", method);
            },
        )
        .await?)
    }
    pub async fn find_config_by_apps(
        &self,
        app_id: &[u64],
        notify_method: &str,
    ) -> AppResult<Vec<AppNotifyConfigModel>> {
        if app_id.is_empty() {
            return Ok(vec![]);
        }
        let method = string_clear(notify_method, StringClear::Ident, Some(65));
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            AppNotifyConfigModel::table_name(),
        ));
        qb.push_where().field_in_copied("app_id", app_id);
        qb.push_and().field_eq("notify_method", method);
        let data = qb
            .build_query_as::<AppNotifyConfigModel>()
            .fetch_all(&self.db)
            .await?;
        Ok(data)
    }
    async fn set_app_config_param_valid(
        &self,
        notify_method: &str,
        call_url: &str,
    ) -> AppResult<()> {
        // 先获取所有字段长度
        let fetch_field = FetchField::new(&self.db);
        let method_max = fetch_field.string_max::<AppNotifyConfigModel>(
            &AppNotifyConfigModel::NOTIFY_METHOD,
        )
        .await
        .len_or(64);
        let url_max = fetch_field.string_max::<AppNotifyConfigModel>(
            &AppNotifyConfigModel::CALL_URL,
        )
        .await
        .len_or(512);

        ValidParam::default()
            .add(
                valid_key!("notify_method"),
                &notify_method,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, method_max))
                    .add_rule(ValidPattern::Ident),
            )
            .add(
                valid_key!("call_url"),
                &call_url,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, url_max))
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
                Update::<_, AppNotifyConfigModel>::new()
                    .set(AppNotifyConfigModel::CALL_URL, call_url.clone())
                    .set(AppNotifyConfigModel::CHANGE_TIME, create_time)
                    .set(AppNotifyConfigModel::CHANGE_USER_ID, change_user_id)
                    .execute(&self.db, |qb| {
                        qb.push_where().field_eq("id", row.id);
                    })
                    .await?;
                row.id
            }
            Err(AppError::Sqlx(sqlx::Error::RowNotFound)) => {
                let notify_method = notify_method.to_owned();
                let res = Insert::<_, AppNotifyConfigModel>::new()
                    .set(AppNotifyConfigModel::APP_ID, app.id)
                    .set(AppNotifyConfigModel::NOTIFY_METHOD, notify_method)
                    .set(AppNotifyConfigModel::CALL_URL, call_url.clone())
                    .set(AppNotifyConfigModel::APP_USER_ID, app.user_id)
                    .set(AppNotifyConfigModel::CHANGE_USER_ID, change_user_id)
                    .set(AppNotifyConfigModel::CREATE_TIME, create_time)
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
        // 先获取所有字段长度
        let fetch_field = FetchField::new(&self.db);
        let method_max = fetch_field.string_max::<AppNotifyDataModel>(
            &AppNotifyDataModel::NOTIFY_METHOD,
        )
        .await
        .len_or(64);
        let key_max = fetch_field.string_max::<AppNotifyDataModel>(&AppNotifyDataModel::NOTIFY_KEY)
            .await
            .len_or(64);
        let payload_max = fetch_field.string_max::<AppNotifyDataModel>(
            &AppNotifyDataModel::NOTIFY_PAYLOAD,
        )
        .await
        .len_or(20000);

        ValidParam::default()
            .add(
                valid_key!("notify_method"),
                &notify_method,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(1, method_max))
                    .add_rule(ValidPattern::Ident),
            )
            .add(
                valid_key!("notify_key"),
                &notify_key,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(0, key_max))
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
                &ValidParamCheck::default().add_rule(ValidStrlen::range(0, payload_max)),
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
                match sqlx::query_scalar::<_, Option<u64>>(&format!(
                    "select max(next_time) from {}
                            where app_id=? and notify_method=? and notify_key=? and status=?",
                    AppNotifyDataModel::table_name(),
                ))
                .bind(app_id)
                .bind(&notify_method)
                .bind(&notify_key)
                .bind(status)
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
        let res = Insert::<_, AppNotifyDataModel>::new()
            .set(AppNotifyDataModel::APP_ID, app_id)
            .set(AppNotifyDataModel::NOTIFY_METHOD, notify_method.clone())
            .set(AppNotifyDataModel::NOTIFY_PAYLOAD, notify_data)
            .set(AppNotifyDataModel::NOTIFY_TYPE, notify_type)
            .set(AppNotifyDataModel::NOTIFY_KEY, notify_key.clone())
            .set(AppNotifyDataModel::STATUS, status)
            .set(AppNotifyDataModel::TRY_MAX, try_max)
            .set(AppNotifyDataModel::TRY_MODE, try_mode)
            .set(AppNotifyDataModel::NEXT_TIME, next_time)
            .set(AppNotifyDataModel::TRY_DELAY, try_delay)
            .set(AppNotifyDataModel::CREATE_TIME, create_time)
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
            match Update::<_, AppNotifyDataModel>::new()
                .set(AppNotifyDataModel::STATUS, del_status)
                .set(AppNotifyDataModel::DELETE_TIME, create_time)
                .execute(&mut *tdb, |qb| {
                    qb.push_where().field_eq("app_id", app_id)
                        .push_and().field_eq("notify_method", notify_method)
                        .push_and().field_eq("notify_key", notify_key)
                        .push_and().field_lt("id", last_id)
                        .push_and().field_eq("status", status);
                })
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
        data: &AppNotifyDataModel,
        del_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<()> {
        // let data = self.find_data_by_id(&notify_id).await?;
        if AppNotifyDataStatus::Delete.eq(data.status) {
            return Ok(());
        }
        let create_time = now_time().unwrap_or_default();
        let del_status = AppNotifyDataStatus::Delete as i8;
        Update::<_, AppNotifyDataModel>::new()
            .set(AppNotifyDataModel::STATUS, del_status)
            .set(AppNotifyDataModel::DELETE_TIME, create_time)
            .execute(&self.db, |qb| {
                qb.push_where().field_eq("id", data.id);
            })
            .await?;
        self.logger
            .add(
                &AppNotifyDataDelLog {
                    source: "del",
                    info: "",
                },
                Some(data.id),
                Some(del_user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    fn data_sql<'a, 'args>(
        &self,
        wb: &mut WhereClause<'a, 'args, MySql>,
        app_id: Option<u64>,
        app_user_id: Option<u64>,
        notify_method: Option<&str>,
        notify_key: Option<&str>,
        status: Option<&[AppNotifyDataStatus]>,
    ) -> Option<()> {
        // Validate inputs first (return None before any push)
        let cleaned_method = notify_method.map(|s| string_clear(s, StringClear::Ident, Some(65)));
        if let Some(ref s) = cleaned_method
            && s.is_empty() {
                return None;
            }
        let status_vals: Option<Vec<i8>> = status.map(|s| s.iter().map(|e| *e as i8).collect());
        if let Some(ref s) = status_vals
            && s.is_empty() {
                return None;
            }

        if let Some(s) = cleaned_method {
            wb.and().field_eq("d.notify_method", s);
        }
        if let Some(s) = notify_key {
            let s = string_clear(s, StringClear::Ident, Some(65));
            wb.and().field_eq("d.notify_key", s);
        }
        if let Some(aid) = app_id {
            wb.and().field_eq("d.app_id", aid);
        }
        if let Some(uid) = app_user_id {
            let qb = wb.and();
            qb.push(format!(
                "d.app_id IN (SELECT app_id FROM {}",
                AppNotifyConfigModel::table_name(),
            ));
            qb.push_where().field_eq("app_user_id", uid);
            qb.push(")");
        }
        if let Some(s) = status_vals {
            let s: Vec<i8> = s.to_vec();
            if s.is_empty() {
                return None;
            }
            wb.and().field_in_copied("d.status", &s);
        }
        Some(())
    }

    //消息数量

    pub async fn data_count(
        &self,
        app_id: Option<u64>,
        app_user_id: Option<u64>,
        notify_method: Option<&str>,
        notify_key: Option<&str>,
        status: Option<&[AppNotifyDataStatus]>,
        total_param: &TotalParam,
    ) -> AppResult<TotalRow> {
        let query = total_param.total_count_query();
        let mut qb = if query.is_threshold_mode() {
            QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from (select 1 from {} as d",
                AppNotifyDataModel::table_name(),
            ))
        } else {
            QueryBuilder::<MySql>::new(format!(
                "select count(*) as total from {} as d",
                AppNotifyDataModel::table_name(),
            ))
        };
        {
            let mut wb = WhereClause::new(&mut qb);
            if self.data_sql(&mut wb, app_id, app_user_id, notify_method, notify_key, status).is_none() {
                return Ok(TotalRow::Exact(0));
            }
        }
        if query.is_threshold_mode() {
            query.push_limit(&mut qb);
            qb.push(") as t");
        }
        let count = qb
            .build_query_scalar::<i64>()
            .fetch_one(&self.db)
            .await? as u64;
        Ok(query.finalize(count))
    }
    //消息列表
    #[allow(clippy::too_many_arguments)]
    pub async fn data_list(
        &self,
        app_id: Option<u64>,
        app_user_id: Option<u64>,
        notify_method: Option<&str>,
        notify_key: Option<&str>,
        status: Option<&[AppNotifyDataStatus]>,
        attr_callback_data: bool,
        limit: &CursorPageParam<u64>,
    ) -> AppResult<(Vec<(AppNotifyDataModel, String)>, CursorPageData<u64>)> {
        let query_limit = limit.page_query("d.id");

        let mut qb = if attr_callback_data {
            QueryBuilder::<MySql>::new(format!(
                "select d.*,c.call_url from {} as d left join {} as c on d.app_id=c.app_id and d.notify_method=c.notify_method",
                AppNotifyDataModel::table_name(),
                AppNotifyConfigModel::table_name(),
            ))
        } else {
            QueryBuilder::<MySql>::new(format!(
                "select d.id,d.app_id,d.notify_method,d.notify_type,d.notify_key,d.status,d.publish_time,d.next_time,d.create_time,'' as call_url from {} as d",
                AppNotifyDataModel::table_name(),
            ))
        };

        let cursor_has = query_limit.has_cursor();
        {
            let mut wb = WhereClause::new(&mut qb);
            if self.data_sql(&mut wb, app_id, app_user_id, notify_method, notify_key, status).is_none() {
                return Ok((vec![], CursorPageData::default()));
            }
            if cursor_has {
                query_limit.push_where(wb.and());
            }
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);

        let mut m_data = qb
            .build()
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

        let next = query_limit.finalize(&mut m_data, |c, d| *d == c.0.id, |c| c.0.id);

        Ok((m_data, next))
    }
}
