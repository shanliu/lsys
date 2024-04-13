use crate::model::{SettingModel, SettingModelRef, SettingStatus, SettingType};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{now_time, RemoteNotify, RequestEnv};
use lsys_logger::dao::ChangeLogger;
use sqlx::{MySql, Pool, Transaction};
use sqlx_model::SqlQuote;
use sqlx_model::{
    executor_option, model_option_set, sql_format, Insert, Select, Update, WhereOption,
};
use std::sync::Arc;

use super::{SettingData, SettingDecode, SettingEncode, SettingLog, SettingResult};
pub struct SingleSetting {
    db: Pool<MySql>,
    logger: Arc<ChangeLogger>,
    //fluent: Arc<FluentBuild>,
    pub(crate) cache: Arc<LocalCache<String, SettingModel>>,
}

impl SingleSetting {
    pub fn new(
        db: Pool<MySql>,
        // _fluent: Arc<FluentBuild>,
        remote_notify: Arc<RemoteNotify>,
        config:LocalCacheConfig,
        logger: Arc<ChangeLogger>,
    ) -> Self {
        Self {
            cache:Arc::from(LocalCache::new(
                remote_notify.clone(),
                config,
            )),
            db,
            logger, //  fluent,
        }
    }
    pub async fn save<'t, T: SettingEncode>(
        &self,
        user_id: &Option<u64>,
        name: &str,
        data: &T,
        change_user_id: &u64,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let name = name.to_owned();
        let edata = data.encode();
        let key = T::key().to_string();
        let time = now_time().unwrap_or_default();
        let uid = user_id.unwrap_or_default();
        let change_user_id = change_user_id.to_owned();
        let user_name_res = Select::type_new::<SettingModel>()
            .fetch_one_by_where::<SettingModel, _>(
                &WhereOption::Where(sql_format!(
                    "setting_type={} and setting_key={} and user_id={} order by id desc",
                    SettingType::Single,
                    key,
                    uid,
                )),
                &self.db,
            )
            .await;
        let did = match user_name_res {
            Err(sqlx::Error::RowNotFound) => {
                let setting_type = SettingType::Single as i8;
                let status = SettingStatus::Enable as i8;
                let new_data = model_option_set!(SettingModelRef,{
                    setting_type:setting_type,
                    setting_key: key,
                    setting_data: edata,
                    user_id: uid,
                    name:name,
                    status: status,
                    change_user_id: change_user_id,
                    change_time: time,
                });
                let dat = executor_option!(
                    {
                        Insert::<sqlx::MySql, SettingModel, _>::new(new_data)
                            .execute(db)
                            .await?
                    },
                    transaction,
                    &self.db,
                    db
                );
                self.cache.clear(&format!("{}-{}", key, uid)).await;
                dat.last_insert_id()
            }
            Ok(set) => {
                let change = sqlx_model::model_option_set!(SettingModelRef,{
                    setting_data: edata,
                    name:name,
                    change_user_id: change_user_id,
                    change_time: time,
                });
                executor_option!(
                    {
                        Update::<sqlx::MySql, SettingModel, _>::new(change)
                            .execute_by_pk(&set, db)
                            .await?;
                    },
                    transaction,
                    &self.db,
                    db
                );
                self.cache
                    .clear(&format!("{}-{}", set.setting_key, set.user_id))
                    .await;
                set.id
            }
            Err(err) => return Err(err.into()),
        };
        self.logger
            .add(
                &SettingLog {
                    action: "single_save",
                    setting_key: key,
                    setting_type: SettingType::Single,
                    name,
                    setting_data: edata,
                },
                &Some(did),
                &Some(uid),
                &Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(did)
    }
    pub async fn find(&self, user_id: &Option<u64>, key: &str) -> SettingResult<SettingModel> {
        let uid = user_id.unwrap_or_default();
        Ok(Select::type_new::<SettingModel>()
            .fetch_one_by_where::<SettingModel, _>(
                &WhereOption::Where(sql_format!(
                    "setting_type={} and setting_key={} and user_id={} order by id desc",
                    SettingType::Single,
                    key,
                    uid,
                )),
                &self.db,
            )
            .await?)
    }
    pub async fn load<T: SettingDecode>(
        &self,
        user_id: &Option<u64>,
    ) -> SettingResult<SettingData<T>> {
        SettingData::try_from(self.find(user_id, T::key()).await?)
    }
}
