use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{Insert, ModelTableName, SqlExpr, Update, WhereOption};
use lsys_core::{db_option_executor, model_option_set, sql_format};
use lsys_core::{now_time, IntoFluentMessage, PageParam, RemoteNotify, RequestEnv};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool, Transaction};
use std::sync::Arc;
use tracing::log::warn;

use super::{SettingData, SettingDecode, SettingEncode, SettingError, SettingLog, SettingResult};
use crate::model::{SettingModel, SettingModelRef, SettingStatus, SettingType};
use lsys_core::db::SqlQuote;

pub struct MultipleSetting {
    db: Pool<MySql>,
    logger: Arc<ChangeLoggerDao>,
    //fluent: Arc<FluentBuild>,
    pub(crate) cache: Arc<LocalCache<String, Vec<SettingModel>>>,
}

impl MultipleSetting {
    pub fn new(
        db: Pool<MySql>,
        // _fluent: Arc<FluentBuild>,
        remote_notify: Arc<RemoteNotify>,
        config: LocalCacheConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(remote_notify.clone(), config)),
            db,
            // fluent,
            logger,
        }
    }
    pub async fn add<T: SettingEncode>(
        &self,
        user_id: Option<u64>,
        name: &str,
        data: &T,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let change_user_id = change_user_id.to_owned();
        let setting_type = SettingType::Multiple as i8;
        let status = SettingStatus::Enable as i8;
        let edata = data.encode();
        let key = T::key().to_string();
        let time = now_time().unwrap_or_default();
        let uid = user_id.unwrap_or_default();
        let name = name.to_owned();
        let new_data = model_option_set!(SettingModelRef,{
            name:name,
            setting_type:setting_type,
            setting_key: key,
            setting_data: edata,
            user_id: uid,
            status: status,
            change_user_id: change_user_id,
            change_time: time,
        });
        let dat = db_option_executor!(
            db,
            {
                Insert::<sqlx::MySql, SettingModel, _>::new(new_data)
                    .execute(db.as_executor())
                    .await?
            },
            transaction,
            &self.db
        );
        self.cache.clear(&format!("{}-{}", key, uid)).await;
        self.logger
            .add(
                &SettingLog {
                    action: "multiple_add",
                    setting_key: &key,
                    user_id: uid,
                    setting_type: SettingType::Multiple,
                    name: &name,
                    setting_data: &edata,
                },
                Some(dat.last_insert_id()),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(dat.last_insert_id())
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn edit<T: SettingEncode>(
        &self,
        user_id: Option<u64>,
        id: u64,
        name: &str,
        data: &T,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let id = id.to_owned();
        let change_user_id = change_user_id.to_owned();
        let name = name.to_owned();
        let edata = data.encode();
        let key = T::key().to_string();
        let time = now_time().unwrap_or_default();
        let change = lsys_core::model_option_set!(SettingModelRef,{
            setting_data: edata,
            name:name,
            change_user_id: change_user_id,
            change_time: time,
        });
        let uid = user_id.unwrap_or_default();

        let cu = db_option_executor!(
            db,
            {
                Update::<sqlx::MySql, SettingModel, _>::new(change)
                    .execute_by_where(
                        &WhereOption::Where(sql_format!(
                            "id={} and setting_type={} and setting_key={} and user_id={}",
                            id,
                            SettingType::Multiple,
                            key,
                            uid,
                        )),
                        db.as_executor(),
                    )
                    .await?
            },
            transaction,
            &self.db
        );

        self.cache.clear(&format!("{}-{}", key, uid)).await;

        self.logger
            .add(
                &SettingLog {
                    action: "multiple_edit",
                    setting_key: &key,
                    user_id: uid,
                    setting_type: SettingType::Multiple,
                    name: &name,
                    setting_data: &edata,
                },
                Some(id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(cu.rows_affected())
    }
    pub async fn del<T: SettingEncode>(
        &self,
        user_id: Option<u64>,
        id: u64,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let change_user_id = change_user_id.to_owned();
        let id = id.to_owned();
        let key = T::key().to_string();
        let status = SettingStatus::Delete as i8;
        let time = now_time().unwrap_or_default();
        let change = lsys_core::model_option_set!(SettingModelRef,{
            status: status,
            change_user_id: change_user_id,
            change_time: time,
        });
        let uid = user_id.unwrap_or_default();

        let data = sqlx::query_as::<_, SettingModel>(&sql_format!(
            "select * from {} where id={} and setting_type={} and setting_key={} and  user_id={}",
            SettingModel::table_name(),
            id,
            SettingType::Multiple,
            key,
            uid
        ))
        .fetch_one(&self.db)
        .await;

        match data {
            Ok(item) => {
                let cu = db_option_executor!(
                    db,
                    {
                        Update::<sqlx::MySql, SettingModel, _>::new(change)
                            .execute_by_pk(&item, db.as_executor())
                            .await?
                    },
                    transaction,
                    &self.db
                );
                self.cache.clear(&format!("{}-{}", key, uid)).await;

                self.logger
                    .add(
                        &SettingLog {
                            action: "multiple_del",
                            setting_key: &item.setting_key,
                            setting_type: SettingType::Multiple,
                            name: &item.name,
                            user_id: uid,
                            setting_data: &item.setting_data,
                        },
                        Some(id),
                        Some(change_user_id),
                        None,
                        env_data,
                    )
                    .await;

                Ok(cu.rows_affected())
            }
            Err(sqlx::Error::RowNotFound) => Ok(0),
            Err(err) => Err(err.into()),
        }
    }
    pub async fn list_data<T: SettingDecode>(
        &self,
        user_id: Option<&[u64]>,
        ids: Option<&[u64]>,
        page: Option<&PageParam>,
    ) -> SettingResult<Vec<SettingData<T>>> {
        let key = T::key().to_string();
        let uid = user_id.unwrap_or_default();
        let mut sql = sql_format!(
            "status={} and setting_type={} and setting_key={} and  user_id={}",
            SettingStatus::Enable as i8,
            SettingType::Multiple as i8,
            key,
            uid
        );
        if let Some(id) = ids {
            if id.is_empty() {
                return Ok(vec![]);
            }
            sql += sql_format!(" and id in ({})", id).as_str();
        }
        if let Some(pdat) = page {
            sql += format!(" limit {} offset {}", pdat.limit, pdat.offset).as_str();
        }

        let data = sqlx::query_as::<_, SettingModel>(&sql_format!(
            "select * from {} where {}",
            SettingModel::table_name(),
            SqlExpr(sql)
        ))
        .fetch_all(&self.db)
        .await?;

        let mut out = Vec::with_capacity(data.len());
        for model in data {
            match SettingData::try_from(model) {
                Ok(dat) => {
                    out.push(dat);
                }
                Err(err) => {
                    warn!(
                        "setting parse fail:{}",
                        err.to_fluent_message().default_format()
                    );
                }
            }
        }
        Ok(out)
    }
    pub async fn list_count<T: SettingDecode>(&self, user_id: Option<u64>) -> SettingResult<i64> {
        let key = T::key().to_string();
        let uid = user_id.unwrap_or_default();
        let sql = sql_format!(
            "select count(*) as total from {} where status={} and setting_type={} and setting_key={} and  user_id={}",
            SettingModel::table_name(),
            SettingStatus::Enable as i8,
            SettingType::Multiple as i8,
            key,
            uid
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    pub async fn find(&self, user_id: Option<u64>, id: u64) -> SettingResult<SettingModel> {
        let id = id.to_owned();
        let uid = user_id.unwrap_or_default();

        Ok(sqlx::query_as::<_, SettingModel>(&sql_format!(
            "select * from {} where id={} and setting_type={} and  user_id={}",
            SettingModel::table_name(),
            id,
            SettingType::Multiple,
            uid
        ))
        .fetch_one(&self.db)
        .await?)
    }
    pub async fn load<T: SettingDecode>(
        &self,
        user_id: Option<u64>,
        id: u64,
    ) -> SettingResult<SettingData<T>> {
        let model = self.find(user_id, id).await?;
        if T::key() != model.setting_key.as_str() {
            return Err(SettingError::Sqlx(sqlx::error::Error::RowNotFound));
        }
        SettingData::try_from(model)
    }
}
