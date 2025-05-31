use std::sync::Arc;

use lsys_core::{db::ModelTableName, sql_format, IntoFluentMessage, TaskNotify};
use sqlx::Pool;
use tracing::{info, warn};

use crate::{
    dao::AppResult,
    model::{AppNotifyConfigModel, AppNotifyTryTimeMode, AppNotifyType},
};
use lsys_core::db::SqlQuote;

use super::AppNotifyRecord;

pub struct AppNotifySender {
    db: Pool<sqlx::MySql>,
    record: Arc<AppNotifyRecord>,
    task_notify: Arc<TaskNotify>,
    notify_method: String,
    notify_type: AppNotifyType,
    try_max: u8,
    try_mode: AppNotifyTryTimeMode,
    try_delay: u16,
    clear_init_status: bool,
}

impl AppNotifySender {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        db: Pool<sqlx::MySql>,
        record: Arc<AppNotifyRecord>,
        task_notify: Arc<TaskNotify>,
        notify_method: impl ToString,
        notify_type: AppNotifyType,
        try_max: u8,
        try_mode: AppNotifyTryTimeMode,
        try_delay: u16,
        clear_init_status: bool,
    ) -> Self {
        Self {
            db,
            record,
            task_notify,
            notify_type,
            notify_method: notify_method.to_string(),
            try_max,
            try_mode,
            try_delay,
            clear_init_status,
        }
    }
    pub async fn send(&self, app_id: u64, notify_key: &str, notify_data: &str) -> AppResult<()> {
        let call_url = match sqlx::query_scalar::<_, String>(&sql_format!(
            "select call_url from {} 
                 where app_id={} and notify_method={} order by id desc limit 1",
            AppNotifyConfigModel::table_name(),
            app_id,
            &self.notify_method,
        ))
        .fetch_one(&self.db)
        .await
        {
            Ok(t) => t,
            Err(sqlx::Error::RowNotFound) => "".to_string(),
            Err(err) => Err(err)?,
        };
        if call_url.trim().is_empty() {
            info!(
                "app_id :{} notify_method:{} not set call_url",
                app_id, &self.notify_method,
            );
            return Ok(());
        }
        self.record
            .add(
                app_id,
                &self.notify_method,
                self.notify_type,
                notify_key,
                notify_data,
                self.try_max,
                self.try_mode,
                self.try_delay,
                self.clear_init_status,
            )
            .await?;
        if let Err(err) = self.task_notify.notify().await {
            warn!(
                "add notify task fail :{}",
                err.to_fluent_message().default_format()
            )
        }
        Ok(())
    }
}
