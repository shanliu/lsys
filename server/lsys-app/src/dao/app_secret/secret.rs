use lsys_core::cache::LocalCache;
use lsys_core::cache::LocalCacheConfig;
use lsys_core::db::Insert;
use lsys_core::db::ModelTableName;
use lsys_core::db::SqlQuote;
use lsys_core::db::Update;
use lsys_core::db::WhereOption;
use lsys_core::now_time;
use lsys_core::rand_str;
use lsys_core::sql_format;
use lsys_core::valid_key;
use lsys_core::RemoteNotify;
use lsys_core::ValidParam;
use lsys_core::ValidParamCheck;
use lsys_core::ValidPattern;
use lsys_core::ValidStrlen;
use serde::Deserialize;
use serde::Serialize;
use sqlx::Acquire;
use sqlx::Executor;
use sqlx::MySql;
use sqlx::Pool;
use sqlx::Transaction;
use std::fmt::Display;
use std::str::FromStr;
use std::sync::Arc;

use crate::dao::AppError;
use crate::dao::AppResult;
use crate::model::AppSecretModel;
use crate::model::AppSecretModelRef;
use crate::model::AppSecretStatus;
use crate::model::AppSecretType;
use lsys_core::model_option_set;

use super::AppSecretRecrod;
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AppSecretCacheKey {
    pub app_id: u64,
    pub secret_type: i8,
}
impl Display for AppSecretCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::json!(self))
    }
}
impl FromStr for AppSecretCacheKey {
    type Err = AppError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(serde_json::from_str::<AppSecretCacheKey>(s)?)
    }
}

pub struct AppSecret {
    db: Pool<MySql>,
    pub(crate) secret_cache: Arc<LocalCache<AppSecretCacheKey, Vec<AppSecretRecrod>>>, //appid,Vec AppSecretModel
}

impl AppSecret {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
    ) -> Self {
        Self {
            db,
            secret_cache: Arc::new(LocalCache::new(remote_notify.clone(), config)),
        }
    }
}

impl AppSecret {
    pub async fn single_set(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
        secret_data: &str,
        time_out: u64,
        user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
    ) -> Result<(), AppError> {
        ValidParam::default()
            .add(
                valid_key!("secret_data"),
                &secret_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(32, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .check()?;
        let mut db = match transaction {
            Some(pb) => pb.begin().await?,
            None => self.db.begin().await?,
        };

        let ntime = now_time().unwrap_or_default();
        let time_out = if time_out > 0 { ntime + time_out } else { 0 };
        let secret_status = AppSecretStatus::Enable as i8;
        let secret_type = secret_type as i8;
        let secret_data = secret_data.to_owned();
        let secret_idata = model_option_set!(AppSecretModelRef,{
            app_id:app_id,
            secret_type:secret_type,
            secret_data:secret_data,
            time_out:time_out,
            status:secret_status,
            add_user_id:user_id,
            change_user_id:user_id,
            change_time:ntime
        });
        let secret_udata = lsys_core::model_option_set!(AppSecretModelRef,{
            secret_data:secret_data,
            status:secret_status,
            time_out:time_out,
            change_user_id:user_id,
            change_time:ntime,
        });
        let res = Insert::<AppSecretModel, _>::new(secret_idata)
            .execute_update(&Update::<AppSecretModel, _>::new(secret_udata), &mut *db)
            .await;
        let add_id = match res {
            Ok(r) => r.last_insert_id(),
            Err(e) => {
                db.rollback().await?;
                return Err(e.into());
            }
        };
        if add_id > 0 {
            let bad_status = AppSecretStatus::Delete as i8;
            let bad_udata = lsys_core::model_option_set!(AppSecretModelRef,{
                status:bad_status,
                change_user_id:user_id,
                change_time:ntime,
            });
            let ures = Update::<AppSecretModel, _>::new(bad_udata)
                .execute_by_where(
                    &WhereOption::Where(sql_format!(
                        "app_id={} and secret_type={} and id!={}",
                        app_id,
                        secret_type,
                        add_id
                    )),
                    &mut *db,
                )
                .await;
            if let Err(e) = ures {
                db.rollback().await?;
                return Err(e.into());
            }
        }
        db.commit().await?;
        self.secret_cache
            .clear(&AppSecretCacheKey {
                app_id,
                secret_type,
            })
            .await;
        Ok(())
    }
    pub async fn multiple_add<'a, E: Executor<'a, Database = sqlx::MySql>>(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
        secret_data: &str,
        time_out: u64,
        user_id: u64,
        db: E,
    ) -> Result<(), AppError> {
        ValidParam::default()
            .add(
                valid_key!("secret_data"),
                &secret_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(32, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .check()?;
        let ntime = now_time().unwrap_or_default();
        let time_out = if time_out > 0 { ntime + time_out } else { 0 };

        let secret_status = AppSecretStatus::Enable as i8;
        let secret_type = secret_type as i8;
        let secret_data = secret_data.to_owned();
        let secret_idata = model_option_set!(AppSecretModelRef,{
            app_id:app_id,
            secret_type:secret_type,
            secret_data:secret_data,
            time_out:time_out,
            status:secret_status,
            add_user_id:user_id,
            change_user_id:user_id,
            change_time:ntime
        });
        let secret_udata = lsys_core::model_option_set!(AppSecretModelRef,{
            secret_data:secret_data,
            status:secret_status,
            time_out:time_out,
            change_user_id:user_id,
            change_time:ntime,
        });
        Insert::<AppSecretModel, _>::new(secret_idata)
            .execute_update(&Update::<AppSecretModel, _>::new(secret_udata), db)
            .await?;
        self.secret_cache
            .clear(&AppSecretCacheKey {
                app_id,
                secret_type,
            })
            .await;
        Ok(())
    }
    pub async fn multiple_del<'a, E: Executor<'a, Database = sqlx::MySql>>(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
        secret_data: &str,
        change_user_id: u64,
        db: E,
    ) -> Result<(), AppError> {
        ValidParam::default()
            .add(
                valid_key!("secret"),
                &secret_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(32, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .check()?;
        let secret_data = secret_data.to_owned();
        let ntime = now_time().unwrap_or_default();
        let status_del = AppSecretStatus::Delete as i8;
        let secret_udata = lsys_core::model_option_set!(AppSecretModelRef,{
            secret_data:secret_data,
            status:status_del,
            change_user_id:change_user_id,
            change_time:ntime,
        });
        Update::<AppSecretModel, _>::new(secret_udata)
            .execute_by_where(
                &WhereOption::Where(sql_format!(
                    "app_id={} and secret_type={} and secret_data={} ",
                    app_id,
                    secret_type as i8,
                    secret_data,
                )),
                db,
            )
            .await?;
        self.secret_cache
            .clear(&AppSecretCacheKey {
                app_id,
                secret_type: secret_type as i8,
            })
            .await;
        Ok(())
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn multiple_change<'a, E: Executor<'a, Database = sqlx::MySql>>(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
        secret_data: &str,
        old_secret_data: &str,
        time_out: u64,
        change_user_id: u64,
        db: E,
    ) -> Result<(), AppError> {
        ValidParam::default()
            .add(
                valid_key!("secret_data"),
                &secret_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(32, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .add(
                valid_key!("old_secret_data"),
                &secret_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(32, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .check()?;
        let ntime = now_time().unwrap_or_default();
        let time_out = if time_out > 0 { ntime + time_out } else { 0 };

        let secret_data = secret_data.to_owned();
        let secret_udata = lsys_core::model_option_set!(AppSecretModelRef,{
            secret_data:secret_data,
            time_out:time_out,
            change_user_id:change_user_id,
            change_time:ntime,
        });
        Update::<AppSecretModel, _>::new(secret_udata)
            .execute_by_where(
                &WhereOption::Where(sql_format!(
                    "app_id={} and secret_type={} and secret_data={} and status={}",
                    app_id,
                    secret_type as i8,
                    old_secret_data,
                    AppSecretStatus::Enable as i8
                )),
                db,
            )
            .await?;
        self.secret_cache
            .clear(&AppSecretCacheKey {
                app_id,
                secret_type: secret_type as i8,
            })
            .await;
        Ok(())
    }

    pub async fn multiple_delete_from_secret<'a, E: Executor<'a, Database = sqlx::MySql>>(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
        secret_data: &str,
        change_user_id: u64,
        db: E,
    ) -> AppResult<()> {
        ValidParam::default()
            .add(
                valid_key!("secret_data"),
                &secret_data,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(32, 64))
                    .add_rule(ValidPattern::Hex),
            )
            .check()?;
        let ntime = now_time().unwrap_or_default();
        let secret_status = AppSecretStatus::Delete as i8;
        let secret_udata = lsys_core::model_option_set!(AppSecretModelRef,{
            status:secret_status,
            change_user_id:change_user_id,
            change_time:ntime,
        });
        Update::<AppSecretModel, _>::new(secret_udata)
            .execute_by_where(
                &WhereOption::Where(sql_format!(
                    "app_id={} and secret_type={} and secret_data={}",
                    app_id,
                    secret_type as i8,
                    secret_data
                )),
                db,
            )
            .await?;
        self.secret_cache
            .clear(&AppSecretCacheKey {
                app_id,
                secret_type: secret_type as i8,
            })
            .await;
        Ok(())
    }
    pub async fn delete_from_app_id<'a, E: Executor<'a, Database = sqlx::MySql>>(
        &self,
        app_id: u64,
        change_user_id: u64,
        db: E,
    ) -> Result<(), AppError> {
        let clear_data = sqlx::query_scalar::<_, i8>(&sql_format!(
            "select secret_type from {} where app_id={} and status={} ",
            AppSecretModel::table_name(),
            app_id,
            AppSecretStatus::Enable as i8,
        ))
        .fetch_all(&self.db)
        .await?;
        let ntime = now_time().unwrap_or_default();
        let secret_status = AppSecretStatus::Delete as i8;
        let secret_udata = lsys_core::model_option_set!(AppSecretModelRef,{
            status:secret_status,
            change_user_id:change_user_id,
            change_time:ntime,
        });
        Update::<AppSecretModel, _>::new(secret_udata)
            .execute_by_where(&WhereOption::Where(sql_format!("app_id={}", app_id)), db)
            .await?;
        for secret_type in clear_data {
            self.secret_cache
                .clear(&AppSecretCacheKey {
                    app_id,
                    secret_type,
                })
                .await;
        }
        Ok(())
    }
}

impl AppSecret {
    //获取指定APP的Secret
    pub async fn multiple_find_secret_by_app_id(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
    ) -> AppResult<Vec<AppSecretRecrod>> {
        let ntime = now_time().unwrap_or_default();
        let out=sqlx::query_as::<_, AppSecretModel>(&sql_format!(
            "select secret_data,time_out from {} where app_id={} and secret_type={} and status={} and (
                time_out=0 or time_out>{}
            )",
            AppSecretModel::table_name(),
            app_id,
            secret_type as i8,
            AppSecretStatus::Enable as i8,
            ntime
        ))
        .fetch_all(&self.db)
        .await
        .map(|e| {
         e.into_iter()
                .flat_map(|e0| if e0.time_out==0||e0.time_out>ntime{Some(AppSecretRecrod::from(e0))}else{None})
                .collect::<Vec<AppSecretRecrod>>()

        })?;
        if out.is_empty() {
            let secret_data = rand_str(lsys_core::RandType::LowerHex, 32);
            self.multiple_add(app_id, secret_type, &secret_data, 0, 0, &self.db)
                .await?;
            return Ok(vec![AppSecretRecrod {
                secret_data,
                time_out: 0,
            }]);
        }
        Ok(out)
    }
    pub async fn single_find_secret_app_id(
        &self,
        app_id: u64,
        secret_type: AppSecretType,
    ) -> AppResult<AppSecretRecrod> {
        let ntime = now_time().unwrap_or_default();
        let out=sqlx::query_as::<_, AppSecretModel>(&sql_format!(
            "select secret_data,time_out from {} where app_id={} and secret_type={} and status={} and (
                time_out=0 or time_out>{}
            ) order by id desc limit 1",
            AppSecretModel::table_name(),
            app_id,
            secret_type as i8,
            AppSecretStatus::Enable as i8,
            ntime
        ))
        .fetch_one(&self.db)
        .await;
        match out {
            Ok(dat) => Ok(AppSecretRecrod::from(dat)),
            Err(err) => {
                if matches!(err, sqlx::Error::RowNotFound) {
                    let secret_data = rand_str(lsys_core::RandType::LowerHex, 32);
                    self.single_set(app_id, secret_type, &secret_data, 0, 0, None)
                        .await?;
                    return Ok(AppSecretRecrod {
                        secret_data,
                        time_out: 0,
                    });
                }
                Err(err.into())
            }
        }
    }
}
