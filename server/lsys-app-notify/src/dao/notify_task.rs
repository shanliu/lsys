use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;

use chrono::{DateTime, Local};
use lsys_app::dao::{App, AppError};

use sqlx::{MySql, Pool};

use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::sql_format;
use lsys_core::{
    now_time, IntoFluentMessage, TaskAcquisition, TaskData, TaskExecutor, TaskItem, TaskRecord,
};
use tracing::warn;

use crate::model::{NotifyDataModel, NotifyDataStatus};

use super::{NotifyError, NotifyRecord};
use lsys_core::db::SqlQuote;

pub struct NotifyTaskItem(NotifyDataModel);
impl TaskItem<u64> for NotifyTaskItem {
    fn to_task_pk(&self) -> u64 {
        self.0.id
    }
}

pub struct NotifyTaskAcquisition {
    db: Pool<MySql>,
}
impl NotifyTaskAcquisition {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
}
#[async_trait]
impl TaskAcquisition<u64, NotifyTaskItem> for NotifyTaskAcquisition {
    //复用父结构体方法实现
    async fn read_send_task(
        &self,
        tasking_record: &HashMap<u64, TaskData>,
        limit: usize,
    ) -> Result<TaskRecord<u64, NotifyTaskItem>, String> {
        let mut sql_vec = vec![];
        sql_vec.push(sql_format!(
            "next_time<={} and status = {}",
            now_time().unwrap_or_default(),
            NotifyDataStatus::Init as i8
        ));
        let ids = tasking_record.keys().copied().collect::<Vec<u64>>();
        if !ids.is_empty() {
            sql_vec.push(sql_format!(" id not in ({})", ids));
        }
        let mut app_res = sqlx::query_as::<_, NotifyDataModel>(&format!(
            "select * from {} where {} order by id asc limit {}",
            NotifyDataModel::table_name(),
            sql_vec.join(" and "),
            limit + 1
        ))
        .fetch_all(&self.db)
        .await
        .map_err(|e| e.to_string())?;
        let next = if app_res.len() > limit {
            app_res.pop();
            true
        } else {
            false
        };
        Ok(TaskRecord::new(
            app_res.into_iter().map(NotifyTaskItem).collect(),
            next,
        ))
    }
}

#[derive(Clone)]
pub struct NotifyTask {
    db: Pool<sqlx::MySql>,
    apps: Arc<App>,
    record: Arc<NotifyRecord>,
    max_try: u16,
}

impl NotifyTask {
    pub fn new(
        db: Pool<sqlx::MySql>,
        apps: Arc<App>,
        record: Arc<NotifyRecord>,
        max_try: u16,
    ) -> Self {
        Self {
            db,
            apps,
            record,
            max_try,
        }
    }
}
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Method, StatusCode,
};

pub const NOTIFY_MIN_DELAY_TIME: u64 = 300;

//每次延迟的时间间隔
fn next_time_add(now_num: &i8) -> u64 {
    match now_num {
        0 => NOTIFY_MIN_DELAY_TIME,
        1 => 30 * 60,
        2 => 60 * 60,
        3 => 6 * 60 * 60,
        4 => 12 * 60 * 60,
        _ => 3600 * 24,
    }
}

async fn change_notify_check_num_error_status(
    db: &Pool<sqlx::MySql>,
    nid: &u64,
    max_try: &u16,
    now_num: &i8,
    msg: &str,
) {
    let ntime = now_time().unwrap_or_default();
    let addtime = next_time_add(now_num);
    let sql = sql_format!(
        r#"UPDATE {}
                SET status={},result={},try_num=try_num+1,next_time=next_time+{},publish_time={}
                WHERE id={};
            "#,
        NotifyDataModel::table_name(),
        SqlExpr(sql_format!(
            "if (try_num>={},{},status)",
            max_try,
            NotifyDataStatus::Fail as i8
        )),
        msg,
        nid,
        addtime,
        ntime,
    );
    if let Err(err) = sqlx::query(sql.as_str()).execute(db).await {
        warn!("change notify data status to fail is fail[{}]{}", nid, err);
    }
}

async fn change_notify_error_status(db: &Pool<sqlx::MySql>, nid: &u64, now_num: &i8, msg: &str) {
    let ntime = now_time().unwrap_or_default();
    let addtime = next_time_add(now_num);
    let sql = sql_format!(
        r#"UPDATE {}
                SET status={},result={} ,next_time=next_time+{},publish_time={}
                WHERE id={};
            "#,
        NotifyDataModel::table_name(),
        NotifyDataStatus::Fail as i8,
        msg,
        nid,
        addtime,
        ntime
    );
    if let Err(err) = sqlx::query(sql.as_str()).execute(db).await {
        warn!("change notify data status fail[{}]{}", nid, err);
    }
}

#[async_trait]
impl TaskExecutor<u64, NotifyTaskItem> for NotifyTask {
    async fn exec(&self, val: NotifyTaskItem) -> Result<(), String> {
        match self
            .record
            .find_config_by_app(val.0.app_id, &val.0.method)
            .await
        {
            Ok(config) => {
                match self
                    .apps
                    .cache()
                    .find_notify_secret_by_app_id(config.app_id)
                    .await
                {
                    Ok(client_secret) => {
                        let mut headers = HeaderMap::new();
                        if let Ok(value) = HeaderValue::from_str("application/json;charset=utf-8") {
                            headers.insert("Content-Type", value);
                        }
                        let now: DateTime<Local> = Local::now();
                        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();
                        let appid = config.app_id.to_string();

                        let mut params = vec![
                            ("app_id", appid.as_str()),
                            ("version", "2.0"),
                            ("timestamp", timestamp.as_str()),
                            ("method", val.0.method.as_str()),
                        ];

                        let mut url_params = url::form_urlencoded::Serializer::new(String::new())
                            .extend_pairs(params.clone())
                            .finish();

                        let payload = val.0.payload.trim();
                        if !payload.is_empty() {
                            url_params += payload;
                        }
                        url_params += client_secret.as_str();
                        let digest = md5::compute(url_params.as_bytes());
                        let hash = format!("{:x}", digest);

                        params.push(("sign", hash.as_str()));

                        let param_str = url::form_urlencoded::Serializer::new(String::new())
                            .extend_pairs(params)
                            .finish();

                        let mut call_url = if config.call_url.contains('?') {
                            config.call_url
                        } else {
                            config.call_url + "?"
                        };
                        if !call_url.ends_with('&') {
                            call_url += "&"
                        }
                        call_url += param_str.as_str();

                        let client = reqwest::Client::builder();
                        let client = client
                            .timeout(Duration::from_secs(2)) //2秒超时
                            .build()
                            .map_err(|e| format!("request client create fail:{}", e))?;

                        let request = client.request(Method::POST, "");

                        match request.send().await {
                            Ok(resp) => {
                                if resp.status() == StatusCode::OK {
                                    let ntime = now_time().unwrap_or_default();
                                    let sql = sql_format!(
                                        r#"UPDATE {}
                                            SET status={},try_num=try_num+1,publish_time={}
                                            WHERE id={};
                                        "#,
                                        NotifyDataModel::table_name(),
                                        NotifyDataStatus::Succ as i8,
                                        &val.0.id,
                                        ntime
                                    );
                                    if let Err(err) =
                                        sqlx::query(sql.as_str()).execute(&self.db).await
                                    {
                                        warn!(
                                            "change notify data status to succ fail[{}]{}",
                                            val.0.id, err
                                        );
                                    }
                                } else {
                                    use futures::StreamExt;
                                    let mut buffer = Vec::new();
                                    let mut stream = resp.bytes_stream().take(256);
                                    while let Some(item) = stream.next().await {
                                        match item {
                                            Ok(st) => {
                                                buffer.extend_from_slice(&st);
                                            }
                                            Err(_) => {
                                                break;
                                            }
                                        }
                                    }
                                    let s = String::from_utf8_lossy(&buffer).to_string();
                                    change_notify_check_num_error_status(
                                        &self.db,
                                        &val.0.id,
                                        &self.max_try,
                                        &val.0.try_num,
                                        &s,
                                    )
                                    .await;
                                }
                            }
                            Err(err) => {
                                change_notify_check_num_error_status(
                                    &self.db,
                                    &val.0.id,
                                    &self.max_try,
                                    &val.0.try_num,
                                    &err.to_string(),
                                )
                                .await;
                                return Err(err.to_string());
                            }
                        };
                    }
                    Err(AppError::Sqlx(sqlx::Error::RowNotFound)) => {
                        change_notify_error_status(&self.db, &val.0.id, &val.0.try_num, "miss app")
                            .await;
                    }
                    Err(err) => return Err(err.to_fluent_message().default_format()),
                };
            }
            Err(NotifyError::Sqlx(sqlx::Error::RowNotFound)) => {
                change_notify_error_status(&self.db, &val.0.id, &val.0.try_num, "miss config")
                    .await;
            }
            Err(err) => return Err(err.to_fluent_message().default_format()),
        };
        Ok(())
    }
}
