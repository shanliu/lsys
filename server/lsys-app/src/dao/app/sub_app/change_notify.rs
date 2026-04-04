use crate::dao::AppNotifySender;
use crate::dao::AppSecret;
use crate::model::AppModel;
use crate::model::AppNotifyDataModel;
use crate::model::AppSecretModel;
use crate::model::AppSecretStatus;
use crate::model::AppSecretType;
use lsys_core::db::{QueryBuilderExt, TableMeta, Update};
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::timeout_task::{TimeOutTaskExec, TimeOutTaskExecutor, TimeOutTaskNextTime};
use lsys_core::utils::now_time;
use serde_json::json;
use sqlx::MySql;
use sqlx::Pool;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tracing::info;
use tracing::warn;

pub struct SubAppChangeNotify {
    db: Pool<MySql>,
    app_notify_sender: AppNotifySender,
    app_secret: Arc<AppSecret>,
}

impl SubAppChangeNotify {
    pub fn new(
        db: Pool<MySql>,
        app_secret: Arc<AppSecret>,
        app_notify_sender: AppNotifySender,
    ) -> Self {
        Self {
            db,
            app_secret,
            app_notify_sender,
        }
    }
    pub(crate) async fn add_app_secret_change_notify(&self, app: &AppModel) {
        if app.parent_app_id == 0 {
            info!("System app Ignore notify:{}", app.id);
            return;
        }
        match self
            .app_secret
            .multiple_find_secret_by_app_id(app.id, AppSecretType::App)
            .await
        {
            Ok(secret) => {
                if let Err(err) = self
                    .app_notify_sender
                    .send(
                        app.parent_app_id,
                        &app.id.to_string(),
                        &json!({
                            "client_id":app.client_id,
                            "sercet_data":secret,
                        })
                        .to_string(),
                    )
                    .await
                {
                    warn!(
                        "add notify data fail:{}",
                        err.to_fluent_message().default_format()
                    );
                }
            }
            Err(e) => {
                warn!(
                    "get app secret fail:{}",
                    e.to_fluent_message().default_format()
                );
            }
        }
    }
}

#[async_trait::async_trait]
impl TimeOutTaskExec for SubAppChangeNotify {
    async fn exec(
        &self,
        max_lock_time: usize,
        mut expire_call: impl FnMut() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send,
    ) -> Result<(), String> {
        let ntime = now_time().unwrap_or_default();
        let mut runtime = ntime;
        let mut start_id = 0;
        loop {
            let add_res = sqlx::query_as::<_, AppModel>(&format!(
                "
                select * from {} as p join (
                    select
                                se.app_id
                        from
                                {} as se
                        left join
                                {} as da
                        on
                                se.app_id= da.app_id
                        where
                                se.status   = ?
                        and  se.time_out >0 and se.time_out <= ?
                        and     se.app_id   > ?
                        and (se.time_out <da.create_time or da.create_time is null)
                        group by
                                se.app_id
                        order by
                                se.app_id asc
                        limit 100
                ) as t on p.id=t.app_id order by id asc limit 100
                ",
                AppModel::table_name(),
                AppSecretModel::table_name(),
                AppNotifyDataModel::table_name()
            ))
            .bind(AppSecretStatus::Enable as i8)
            .bind(ntime)
            .bind(start_id)
            .fetch_all(&self.db)
            .await
            .map_err(|e| e.to_string())?; 
            if add_res.is_empty() {
                break;
            }
            for app_item in add_res {
                start_id = app_item.id;
                self.add_app_secret_change_notify(&app_item).await;
                let status = AppSecretStatus::Delete as i8;
                Update::<_, AppSecretModel>::new()
                    .set(AppSecretModel::STATUS, status)
                    .set(AppSecretModel::CHANGE_USER_ID, 0u64)
                    .set(AppSecretModel::CHANGE_TIME, ntime)
                    .execute(&self.db, |qb| {
                        qb.push_where()
                            .field_eq("app_id", start_id)
                            .push_and()
                            .field_eq("status", AppSecretStatus::Enable as i8)
                            .push_and()
                            .push("time_out>0")
                            .push_and()
                            .field_lte("time_out", ntime);
                    })
                    .await
                    .map_err(|e| e.to_string())?;
            }
            let last_now_time = now_time().unwrap_or_default();
            if (last_now_time - runtime) > (max_lock_time as u64) {
                return Err(format!(
                    "app change notify timeout[last run time:{},start time:{}]",
                    last_now_time, runtime
                ));
            }
            if (last_now_time - runtime) * 2 > (max_lock_time as u64) {
                //时间小于一半延长一次有效期
                expire_call().await;
            }
            runtime = last_now_time;
        }
        Ok(())
    }
}
#[async_trait::async_trait]
impl TimeOutTaskNextTime for SubAppChangeNotify {
    async fn next_time(&self, max_lock_time: usize) -> Result<Option<u64>, String> {
        let ntime = now_time().unwrap_or_default();
        let timeout_res = sqlx::query_scalar::<_, u64>(&format!(
            r#"
                select
                    se.time_out
                from
                        {} as se
                left join
                        {} as da
                on
                        se.app_id= da.app_id
                where
                        se.status   = ?
                and  se.time_out >0 and se.time_out <= ?
                and (se.time_out <da.create_time or da.create_time is null)
                order by
                        se.time_out asc
                limit 1
            "#,
            AppSecretModel::table_name(),
            AppNotifyDataModel::table_name()
        ))
        .bind(AppSecretStatus::Enable as i8)
        .bind(ntime + max_lock_time as u64)
        .fetch_one(&self.db)
        .await; 
        match timeout_res {
            Ok(dat) => Ok(Some(dat)),
            Err(sqlx::Error::RowNotFound) => Ok(None),
            Err(err) => Err(err.to_string()),
        }
    }
}
#[async_trait::async_trait]
impl TimeOutTaskExecutor for SubAppChangeNotify {
    type Exec = Self;
    type NextTime = Self;
}
