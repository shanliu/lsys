use crate::model::{SettingModel, SettingModelRef, SettingStatus, SettingType};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{Insert, ModelTableName, SqlQuote, Update};
use lsys_core::{
    db_option_executor, model_option_set, sql_format, string_clear, valid_key, StringClear,
    ValidParam, ValidParamCheck, ValidPattern, ValidStrlen, STRING_CLEAR_FORMAT,
};
use lsys_core::{now_time, RemoteNotify, RequestEnv};
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool, Transaction};
use std::sync::Arc;

use super::{SettingData, SettingDecode, SettingEncode, SettingLog, SettingResult};
pub struct SingleSetting {
    db: Pool<MySql>,
    logger: Arc<ChangeLoggerDao>,
    //fluent: Arc<FluentBuild>,
    pub(crate) cache: Arc<LocalCache<String, SettingModel>>,
}

impl SingleSetting {
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
            logger, //  fluent,
        }
    }
}
pub struct SingleSettingData<'t, T: SettingEncode> {
    pub name: &'t str,
    pub data: &'t T,
}
impl SingleSetting {
    async fn save_param_valid(&self, key: &str, name: &str, data: &str) -> SettingResult<()> {
        ValidParam::default()
            .add(
                valid_key!("setting_key"),
                &key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .add(
                valid_key!("setting_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 32)),
            )
            .add(
                valid_key!("setting_data"),
                &data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 60000)),
            )
            .check()?;
        Ok(())
    }
    pub async fn save<T: SettingEncode>(
        &self,
        user_id: Option<u64>,
        param: &SingleSettingData<'_, T>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let name = param.name.to_owned();
        let edata = param.data.encode();
        let key = T::key().to_string();
        self.save_param_valid(&key, &name, &edata).await?;

        let time = now_time().unwrap_or_default();
        let uid = user_id.unwrap_or_default();
        let change_user_id = change_user_id.to_owned();

        let user_name_res = sqlx::query_as::<_, SettingModel>(&sql_format!(
            "select * from {} where setting_type={} and setting_key={} and user_id={} order by id desc",
            SettingModel::table_name(),
            SettingType::Single,
            key,
            uid,
        ))
        .fetch_one(&self.db)
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
                let dat = db_option_executor!(
                    db,
                    {
                        Insert::<SettingModel, _>::new(new_data)
                            .execute(db.as_executor())
                            .await?
                    },
                    transaction,
                    &self.db
                );
                self.cache.clear(&format!("{}-{}", key, uid)).await;
                dat.last_insert_id()
            }
            Ok(set) => {
                let change = lsys_core::model_option_set!(SettingModelRef,{
                    setting_data: edata,
                    name:name,
                    change_user_id: change_user_id,
                    change_time: time,
                });
                db_option_executor!(
                    db,
                    {
                        Update::<SettingModel, _>::new(change)
                            .execute_by_pk(&set, db.as_executor())
                            .await?;
                    },
                    transaction,
                    &self.db
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
                    setting_key: &key,
                    setting_type: SettingType::Single,
                    name: &name,
                    user_id: uid,
                    setting_data: &edata,
                },
                Some(did),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;
        Ok(did)
    }
    pub async fn find(&self, user_id: Option<u64>, key: &str) -> SettingResult<SettingModel> {
        let uid = user_id.unwrap_or_default();
        let key = string_clear(key, StringClear::Option(STRING_CLEAR_FORMAT), Some(33));
        Ok(sqlx::query_as::<_, SettingModel>(&sql_format!(
            "select * from {} where setting_type={} and setting_key={} and user_id={} order by id desc",
            SettingModel::table_name(),
            SettingType::Single,
            key,
            uid,
        ))
        .fetch_one(&self.db)
        .await?)
    }
    pub async fn load<T: SettingDecode>(
        &self,
        user_id: Option<u64>,
    ) -> SettingResult<SettingData<T>> {
        SettingData::try_from(self.find(user_id, T::key()).await?)
    }
}
