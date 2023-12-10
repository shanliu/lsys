use std::sync::Arc;
use std::time::Duration;

use crate::model::{
    NotifyConfigModel, NotifyConfigModelRef, NotifyDataModel, NotifyDataModelRef, NotifyDataStatus,
};
use lsys_core::{now_time, LimitParam, RequestEnv};

use reqwest::Method;
use serde::Serialize;
use sqlx::Pool;
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, SqlExpr, Update};

use sqlx_model::SqlQuote;
use tracing::warn;

use super::{NotifyError, NotifyResult};

pub struct NotifyRecord {
    db: Pool<sqlx::MySql>,
    logger: Arc<ChangeLogger>,
}

impl NotifyRecord {
    pub fn new(db: Pool<sqlx::MySql>, logger: Arc<ChangeLogger>) -> Self {
        Self { db, logger }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_data_by_id,
        u64,
        NotifyDataModel,
        NotifyResult<NotifyDataModel>,
        id,
        "id={id}"
    );

    pub async fn find_config_by_app(
        &self,
        app_id: &u64,
        method: &str,
    ) -> NotifyResult<NotifyConfigModel> {
        let data = sqlx_model::Select::type_new::<NotifyConfigModel>()
            .fetch_one_by_where::<NotifyConfigModel, _>(
                &sqlx_model::WhereOption::Where(sqlx_model::sql_format!(
                    "app_id={} and method={}",
                    app_id,
                    method
                )),
                &self.db,
            )
            .await?;
        Ok(data)
    }
    pub async fn find_config_by_apps(
        &self,
        app_id: &[u64],
        method: &str,
    ) -> NotifyResult<Vec<NotifyConfigModel>> {
        if app_id.is_empty() {
            return Ok(vec![]);
        }
        let data = sqlx_model::Select::type_new::<NotifyConfigModel>()
            .fetch_all_by_where::<NotifyConfigModel, _>(
                &sqlx_model::WhereOption::Where(sqlx_model::sql_format!(
                    "app_id in ({}) and method={}",
                    app_id,
                    method
                )),
                &self.db,
            )
            .await?;
        Ok(data)
    }

    pub async fn set_app_config(
        &self,
        app_id: &u64,
        method: &str,
        call_url: &str,
        change_user_id: &u64,
        env_data: Option<&RequestEnv>,
    ) -> NotifyResult<u64> {
        if !call_url.starts_with("http://") && !call_url.starts_with("https://") {
            return Err(NotifyError::System(
                "your submit notify not support".to_string(),
            ));
        }
        let client = reqwest::Client::builder();
        let client = client
            .timeout(Duration::from_secs(5))
            .build()
            .map_err(|e| NotifyError::System(e.to_string()))?;
        client
            .request(Method::POST, call_url)
            .send()
            .await
            .map_err(|e| NotifyError::System(e.to_string()))?;

        let call_url = call_url.to_owned();
        let change_user_id = change_user_id.to_owned();
        let create_time = now_time().unwrap_or_default();
        let id = match self.find_config_by_app(app_id, method).await {
            Ok(row) => {
                let change = sqlx_model::model_option_set!(NotifyConfigModelRef,{
                    call_url:call_url,
                    change_time:create_time,
                    change_user_id:change_user_id,
                });
                Update::<sqlx::MySql, NotifyConfigModel, _>::new(change)
                    .execute_by_pk(&row, &self.db)
                    .await?;
                row.id
            }
            Err(NotifyError::Sqlx(sqlx::Error::RowNotFound)) => {
                let method = method.to_owned();
                let res = Insert::<sqlx::MySql, NotifyConfigModel, _>::new(
                    model_option_set!(NotifyConfigModelRef ,{
                        app_id: *app_id,
                        method: method,
                        call_url:call_url,
                        user_id:change_user_id,
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
                    method: method.to_owned(),
                    url: call_url.to_owned(),
                },
                &Some(id),
                &Some(change_user_id),
                &Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(id)
    }

    pub async fn add(&self, method: &str, app_id: &u64, data: &str) -> NotifyResult<u64> {
        let method = method.to_owned();
        let payload = data.to_owned();
        let create_time = now_time().unwrap_or_default();
        let status = NotifyDataStatus::Init as i8;
        let res =
            Insert::<sqlx::MySql, NotifyDataModel, _>::new(model_option_set!(NotifyDataModelRef ,{
                app_id: *app_id,
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
        app_id: &Option<u64>,
        method: &Option<String>,
        status: &Option<NotifyDataStatus>,
    ) -> NotifyResult<i64> {
        let mut sqlwhere = vec![];
        if let Some(s) = method {
            sqlwhere.push(sql_format!("method={}", s));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }

        if let Some(s) = status {
            sqlwhere.push(sql_format!("status={} ", *s));
        }

        let sql = sql_format!(
            "select count(*) as total from {}  {}",
            NotifyDataModel::table_name(),
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
        app_id: &Option<u64>,
        method: &Option<String>,
        status: &Option<NotifyDataStatus>,
        limit: &Option<LimitParam>,
    ) -> NotifyResult<(Vec<NotifyDataModel>, Option<u64>)> {
        let mut sqlwhere = vec![];
        if let Some(s) = method {
            sqlwhere.push(sql_format!("method={}", s));
        }
        if let Some(aid) = app_id {
            sqlwhere.push(sql_format!("app_id = {}  ", aid));
        }

        if let Some(s) = status {
            sqlwhere.push(sql_format!("status={} ", *s));
        }

        let where_sql = if let Some(page) = limit {
            format!(
                "{} {} {} order by {} {} ",
                if sqlwhere.is_empty() { "where " } else { "" },
                sqlwhere.join(" and "),
                page.where_sql("id", Some("and")),
                page.order_sql("id"),
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
            "select m.* from {}   {}",
            NotifyDataModel::table_name(),
            where_sql
        );

        let res = sqlx::query_as::<_, NotifyDataModel>(sql.as_str());
        let mut m_data = res.fetch_all(&self.db).await?;

        let next = limit
            .as_ref()
            .map(|page| page.tidy(&mut m_data))
            .unwrap_or_default();

        Ok((m_data, next.map(|t| t.id)))
    }
}

use lsys_logger::dao::{ChangeLogData, ChangeLogger};
#[derive(Serialize)]
pub(crate) struct AppNotifyConfigLog {
    pub method: String,
    pub url: String,
}

impl ChangeLogData for AppNotifyConfigLog {
    fn log_type<'t>() -> &'t str {
        "app-notify-set"
    }
    fn message(&self) -> String {
        format!("set {} notify url", self.method,)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
