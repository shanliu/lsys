use std::sync::Arc;
use std::time::Duration;

use crate::dao::{AppError, AppResult};
use crate::model::{
    AppModel, AppNotifyConfigModel, AppNotifyConfigModelRef, AppNotifyDataModel,
    AppNotifyDataModelRef, AppNotifyDataStatus,
};
use lsys_core::{fluent_message, now_time, LimitParam, RequestEnv};

use lsys_core::db::{Insert, ModelTableName, SqlExpr, Update};
use lsys_core::{model_option_set, sql_format};
use reqwest::Method;
use serde::Serialize;
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
        method: &str,
    ) -> AppResult<AppNotifyConfigModel> {
        let data = sqlx::query_as::<_, AppNotifyConfigModel>(&sql_format!(
            "select * from {} where app_id={} and method={}",
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
        method: &str,
    ) -> AppResult<Vec<AppNotifyConfigModel>> {
        if app_id.is_empty() {
            return Ok(vec![]);
        }
        let data = sqlx::query_as::<_, AppNotifyConfigModel>(&sql_format!(
            "select * from {} where app_id in ({}) and method={}",
            AppNotifyConfigModel::table_name(),
            app_id,
            method
        ))
        .fetch_all(&self.db)
        .await?;
        Ok(data)
    }

    pub async fn set_app_config(
        &self,
        app: &AppModel,
        method: &str,
        call_url: &str,
        change_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> AppResult<u64> {
        if !call_url.starts_with("http://") && !call_url.starts_with("https://") {
            return Err(AppError::System(fluent_message!("notify-call-not-support")));
        }
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
        let id = match self.find_config_by_app(app.id, method).await {
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
                let method = method.to_owned();
                let res = Insert::<AppNotifyConfigModel, _>::new(
                    model_option_set!(AppNotifyConfigModelRef ,{
                        app_id: app.id,
                        method: method,
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
                    method,
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

    pub async fn add(&self, method: &str, app_id: u64, data: &str) -> AppResult<u64> {
        let method = method.to_owned();
        let payload = data.to_owned();
        let create_time = now_time().unwrap_or_default();
        let status = AppNotifyDataStatus::Init as i8;
        let res = Insert::<AppNotifyDataModel, _>::new(model_option_set!(AppNotifyDataModelRef ,{
            app_id:app_id,
            method: method,
            payload: payload,
            status: status,
            try_num: 0,
            create_time: create_time,
        }))
        .execute(&self.db)
        .await
        .map_err(|e| {
            warn!("add notify error fail:{}", e);
            e
        })?;
        Ok(res.last_insert_id())
    }

    //消息数量
    pub async fn data_count(
        &self,
        app_id: Option<u64>,
        app_user_id: Option<u64>,
        method: Option<&str>,
        status: Option<AppNotifyDataStatus>,
    ) -> AppResult<i64> {
        let mut sqlwhere = vec![];
        if let Some(s) = method {
            sqlwhere.push(sql_format!("method={}", s));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }
        if let Some(uid) = app_user_id {
            sqlwhere.push(sql_format!(
                "app_id in ( select app_id from {} where app_user_id = {} )",
                AppNotifyConfigModel::table_name(),
                uid
            ));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("status={} ", s));
        }

        let sql = sql_format!(
            "select count(*) as total from {}  {}",
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
        method: Option<&str>,
        status: Option<AppNotifyDataStatus>,
        limit: Option<&LimitParam>,
    ) -> AppResult<(Vec<(AppNotifyDataModel, String)>, Option<u64>)> {
        let mut sqlwhere = vec![];
        if let Some(s) = method {
            sqlwhere.push(sql_format!("d.method={}", s));
        }
        if let Some(uid) = app_user_id {
            sqlwhere.push(sql_format!("c.app_user_id = {}  ", uid));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("d.app_id = {}  ", aid));
        }
        if let Some(s) = status {
            sqlwhere.push(sql_format!("d.status={} ", s));
        }

        let where_sql = if let Some(page) = limit {
            format!(
                "{} {} {} order by {} {} ",
                if sqlwhere.is_empty() { "where " } else { "" },
                sqlwhere.join(" and "),
                page.where_sql("d.id", Some("and")),
                page.order_sql("d.id"),
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
            "select d.*,c.call_url from {} as d join {} as c on d.app_id=c.app_id and d.method=c.method
            {}",
            AppNotifyDataModel::table_name(),
            AppNotifyConfigModel::table_name(),
            where_sql
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

use lsys_logger::dao::{ChangeLogData, ChangeLoggerDao};
#[derive(Serialize)]
pub(crate) struct AppNotifyConfigLog<'t> {
    pub method: &'t str,
    pub url: &'t str,
    pub user_id: u64,
}

impl ChangeLogData for AppNotifyConfigLog<'_> {
    fn log_type() -> &'static str {
        "app-notify-set"
    }
    fn message(&self) -> String {
        format!("set {} notify url", self.method,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
