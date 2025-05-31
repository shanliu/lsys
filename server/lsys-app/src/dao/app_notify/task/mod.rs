mod request_reqwest;
mod task_acquisition;
use crate::dao::{App, AppSecret, AppSecretRecrod};
use crate::model::{
    AppModel, AppNotifyConfigModel, AppNotifyDataModel, AppNotifyDataStatus, AppNotifyTryTimeMode,
    AppNotifyType, AppSecretType,
};
use async_trait::async_trait;
use lsys_core::db::SqlQuote;
use lsys_core::db::{ModelTableName, SqlExpr};
use lsys_core::{now_time, IntoFluentMessage, TaskExecutor, TaskNotify, ValidRule};
use lsys_core::{sql_format, ValidUrl};
use sqlx::Pool;
use std::sync::Arc;
use tracing::{debug, info, warn};

pub use request_reqwest::*;
pub use task_acquisition::*;

#[async_trait]
pub trait AppNotifyRequest: Send + Sync + 'static {
    async fn exec_request(
        &self,
        app: &AppModel,
        config: &AppNotifyConfigModel,
        record: &AppNotifyDataModel,
        secret: &AppSecretRecrod,
    ) -> Result<(), String>;
}

#[derive(Clone)]
pub enum AppNotifyRetryTimeMode {
    Fixed(u16),
    Exponential(u16),
}

pub struct AppNotifyTask {
    db: Pool<sqlx::MySql>,
    app: Arc<App>,
    app_secret: Arc<AppSecret>,
    record: Arc<AppNotifyRecord>,
    task_notify: Arc<TaskNotify>,
    request_box: Vec<(AppNotifyType, Box<dyn AppNotifyRequest>)>,
}

impl AppNotifyTask {
    pub fn new(
        db: Pool<sqlx::MySql>,
        app: Arc<App>,
        app_secret: Arc<AppSecret>,
        record: Arc<AppNotifyRecord>,
        task_notify: Arc<TaskNotify>,
        request_box: Vec<(AppNotifyType, Box<dyn AppNotifyRequest>)>,
    ) -> Self {
        Self {
            db,
            app,
            app_secret,
            record,
            task_notify,
            request_box,
        }
    }
}

use super::AppNotifyRecord;

pub const NOTIFY_MIN_DELAY_TIME: u64 = 30; //最小回调间隔

//每次延迟的时间间隔
fn next_time_add(now_num: u8, retry_mode: i8, retry_base: u16) -> u64 {
    let retry_time = if AppNotifyTryTimeMode::Exponential.eq(retry_mode) {
        (retry_base as u64) * 2u64.pow(now_num as u32)
    } else {
        retry_base as u64
    };
    if retry_time < NOTIFY_MIN_DELAY_TIME {
        NOTIFY_MIN_DELAY_TIME
    } else if retry_time > 3600 * 24 {
        3600 * 24
    } else {
        retry_time
    }
}

async fn change_notify_check_num_error_status(
    db: &Pool<sqlx::MySql>,
    nid: u64,
    now_num: u8,
    retry_mode: i8,
    retry_base: u16,
    msg: &str,
) {
    let ntime = now_time().unwrap_or_default();
    let addtime = next_time_add(now_num, retry_mode, retry_base);

    match db.begin().await {
        Ok(mut tdb) => {
            let sql = sql_format!(
                r#"
                UPDATE {} t1
                JOIN (
                    SELECT app_id, notify_method, notify_key 
                    FROM {}
                    WHERE id = {}
                ) t2 ON t1.app_id = t2.app_id 
                    AND t1.notify_method = t2.notify_method 
                    AND t1.notify_key = t2.notify_key
                SET t1.next_time = {}
                WHERE t1.id != {} AND t1.status = {};
                
            "#,
                AppNotifyDataModel::table_name(),
                AppNotifyDataModel::table_name(),
                nid,
                ntime + addtime,
                nid,
                AppNotifyDataStatus::Init as i8,
            );
            if let Err(err) = sqlx::query(sql.as_str()).execute(&mut *tdb).await {
                let _ = tdb.rollback().await;
                warn!("change notify data other record fail[{}]{}", nid, err);
                return;
            }
            let sql = sql_format!(
                r#"UPDATE {}
                SET status={},result={},try_num=try_num+1,next_time={},publish_time={}
                WHERE id={};
            "#,
                AppNotifyDataModel::table_name(),
                SqlExpr(sql_format!(
                    "if ((try_num+1)>=try_max,{},status)",
                    AppNotifyDataStatus::Fail as i8
                )),
                msg,
                ntime + addtime,
                ntime,
                nid,
            );
            if let Err(err) = sqlx::query(sql.as_str()).execute(&mut *tdb).await {
                warn!("change notify data status to fail is fail[{}]{}", nid, err);
            }
            if let Err(db_err) = tdb.commit().await {
                warn!("change notify commit Transaction fail[{}]{}", nid, db_err);
            }
        }
        Err(db_err) => {
            warn!("change notify begin Transaction fail[{}]{}", nid, db_err);
        }
    }
}

async fn change_notify_error_status(
    db: &Pool<sqlx::MySql>,
    nid: u64,
    now_num: u8,
    retry_mode: i8,
    retry_base: u16,
    msg: &str,
) {
    let ntime = now_time().unwrap_or_default();
    let addtime = next_time_add(now_num, retry_mode, retry_base);

    let sql = sql_format!(
        r#"
          UPDATE {} AS t1
            JOIN (
                SELECT app_id, notify_method, notify_key
                FROM {}
                WHERE id = {}
            ) AS t2 ON t1.app_id = t2.app_id
                    AND t1.notify_method = t2.notify_method
                    AND t1.notify_key = t2.notify_key
            SET
                t1.status = {},
                t1.result = {},
                t1.next_time = {},  
                t1.publish_time = {} 
            WHERE
                t1.status = {}
        "#,
        AppNotifyDataModel::table_name(),
        AppNotifyDataModel::table_name(),
        nid,
        AppNotifyDataStatus::Fail as i8,
        msg,
        ntime + addtime,
        ntime,
        AppNotifyDataStatus::Init as i8,
    );
    if let Err(err) = sqlx::query(sql.as_str()).execute(db).await {
        warn!("change notify data status fail[{}]{}", nid, err);
    }
}

impl AppNotifyTask {
    async fn _exec(&self, val: AppAppNotifyTaskItem) -> Result<(), String> {
        let app = match self.app.cache().find_by_id(val.0.app_id).await {
            Ok(val) => val,
            Err(err) => {
                let msg = err.to_fluent_message().default_format();
                change_notify_error_status(
                    &self.db,
                    val.0.id,
                    val.0.try_num,
                    val.0.try_mode,
                    val.0.try_delay,
                    &msg,
                )
                .await;
                return Err(msg);
            }
        };
        let config = match self
            .record
            .find_config_by_app(val.0.app_id, &val.0.notify_method)
            .await
        {
            Ok(val) => val,
            Err(err) => {
                let msg = err.to_fluent_message().default_format();
                change_notify_error_status(
                    &self.db,
                    val.0.id,
                    val.0.try_num,
                    val.0.try_mode,
                    val.0.try_delay,
                    &msg,
                )
                .await;
                return Err(msg);
            }
        };
        if ValidUrl::default().check(&config.call_url).is_err() {
            info!(
                "notify config {} call url is bad :{}",
                config.id, &config.call_url,
            );
            change_notify_error_status(
                &self.db,
                val.0.id,
                val.0.try_num,
                val.0.try_mode,
                val.0.try_delay,
                "bad call url",
            )
            .await;
            return Err("bad call url".to_string());
        }
        let client_secret = match self
            .app_secret
            .cache()
            .single_find_secret_app_id(config.app_id, AppSecretType::Notify)
            .await
        {
            Ok(val) => val,
            Err(err) => {
                let msg = err.to_fluent_message().default_format();
                change_notify_error_status(
                    &self.db,
                    val.0.id,
                    val.0.try_num,
                    val.0.try_mode,
                    val.0.try_delay,
                    &msg,
                )
                .await;
                return Err(msg);
            }
        };

        let exec = match self.request_box.iter().find(|e| e.0.eq(val.0.notify_type)) {
            Some(val) => val,
            None => {
                change_notify_error_status(
                    &self.db,
                    val.0.id,
                    val.0.try_num,
                    val.0.try_mode,
                    val.0.try_delay,
                    "notify type support",
                )
                .await;
                return Ok(());
            }
        };
        match exec
            .1
            .exec_request(&app, &config, &val.0, &client_secret)
            .await
        {
            Ok(()) => {
                debug!("notify {} success", &val.0.id);
                let ntime = now_time().unwrap_or_default();
                let sql = sql_format!(
                    r#"UPDATE {}
                        SET status={},try_num=try_num+1,publish_time={}
                        WHERE id={};
                    "#,
                    AppNotifyDataModel::table_name(),
                    AppNotifyDataStatus::Succ as i8,
                    ntime,
                    &val.0.id,
                );
                if let Err(err) = sqlx::query(sql.as_str()).execute(&self.db).await {
                    warn!(
                        "change notify data status to succ fail[{}]{}",
                        val.0.id, err
                    );
                }
            }
            Err(err_msg) => {
                info!("notify {} fail", &val.0.id);
                change_notify_check_num_error_status(
                    &self.db,
                    val.0.id,
                    val.0.try_num,
                    val.0.try_mode,
                    val.0.try_delay,
                    &err_msg,
                )
                .await;
                return Err(err_msg);
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TaskExecutor<u64, AppAppNotifyTaskItem> for AppNotifyTask {
    async fn exec(&self, val: AppAppNotifyTaskItem) -> Result<(), String> {
        let task_id = val.0.id;
        let res = self._exec(val).await;
        let sql = sql_format!(
            r#"select id from {}
                WHERE (app_id, notify_method, notify_key) = (
                    SELECT app_id, notify_method, notify_key 
                    FROM {} 
                    WHERE id = {} and status in ({})
                ) 
                AND id != {} AND status={} limit 1
            "#,
            AppNotifyDataModel::table_name(),
            AppNotifyDataModel::table_name(),
            task_id,
            [
                AppNotifyDataStatus::Succ as i8,
                AppNotifyDataStatus::Fail as i8,
            ],
            task_id,
            AppNotifyDataStatus::Init as i8,
        );
        if sqlx::query_scalar::<_, u64>(&sql)
            .fetch_one(&self.db)
            .await
            .is_ok()
        {
            if let Err(e) = self.task_notify.notify().await {
                info!(
                    "notify next app fail:{} on :{}",
                    e.to_fluent_message().default_format(),
                    task_id
                );
            }
        }
        res
    }
}
