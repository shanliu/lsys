use crate::model::{SettingModel, SettingStatus, SettingType};
use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::{
    Insert, OptionTxExecutor, QueryBuilderExt, TableMeta, Update, utils::FetchField,
};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{RequestEnv, STRING_CLEAR_FORMAT, StringClear, now_time, string_clear};
use lsys_core::valid_key;
use lsys_core::valid_param::{ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
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
        let fetch_field = FetchField::new(&self.db);
        let setting_key_max = fetch_field
            .string_max::<SettingModel>(&SettingModel::SETTING_KEY)
            .await
            .len_or(32);
        let name_max = fetch_field
            .string_max::<SettingModel>(&SettingModel::NAME)
            .await
            .len_or(32);
        let setting_data_max = fetch_field
            .string_max::<SettingModel>(&SettingModel::SETTING_DATA)
            .await
            .len_or(60000);

        ValidParam::default()
            .add(
                valid_key!("setting_key"),
                &key,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Ident)
                    .add_rule(ValidStrlen::range(1, setting_key_max)),
            )
            .add(
                valid_key!("setting_name"),
                &name,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, name_max)),
            )
            .add(
                valid_key!("setting_data"),
                &data,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, setting_data_max)),
            )
            .check()?;
        Ok(())
    }
    pub async fn save<T: SettingEncode>(
        &self,
        user_id: Option<u64>,
        param: &SingleSettingData<'_, T>,
        change_user_id: u64,
        mut transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let name = param.name.to_owned();
        let edata = param.data.encode();
        let key = T::key().to_string();
        self.save_param_valid(&key, &name, &edata).await?;

        let time = now_time().unwrap_or_default();
        let uid = user_id.unwrap_or_default();
        let change_user_id = change_user_id.to_owned();

        let tmp_res = sqlx::query_as::<_, SettingModel>(&format!(
            "select * from {} where setting_type=? and setting_key=? and user_id=? order by id desc",
            SettingModel::table_name(),
        ))
        .bind(SettingType::Single as i8)
        .bind(&key)
        .bind(uid)
        .fetch_one(&self.db)
        .await;

        let did = match tmp_res {
            Err(sqlx::Error::RowNotFound) => {
                let setting_type = SettingType::Single as i8;
                let status = SettingStatus::Enable as i8;
                let dat = Insert::<_, SettingModel>::new()
                    .set(SettingModel::SETTING_TYPE, setting_type)
                    .set(SettingModel::SETTING_KEY, &key)
                    .set(SettingModel::SETTING_DATA, &edata)
                    .set(SettingModel::USER_ID, uid)
                    .set(SettingModel::NAME, &name)
                    .set(SettingModel::STATUS, status)
                    .set(SettingModel::CHANGE_USER_ID, change_user_id)
                    .set(SettingModel::CHANGE_TIME, time)
                    .execute(OptionTxExecutor::new(transaction.as_deref_mut(), &self.db))
                    .await?;
                self.cache.clear(&format!("{}-{}", key, uid)).await;
                dat.last_insert_id()
            }
            Ok(set) => {
                Update::<_, SettingModel>::new()
                    .set(SettingModel::SETTING_DATA, &edata)
                    .set(SettingModel::NAME, &name)
                    .set(SettingModel::CHANGE_USER_ID, change_user_id)
                    .set(SettingModel::CHANGE_TIME, time)
                    .execute(OptionTxExecutor::new(transaction, &self.db), |qb| {
                        qb.push_where().field_eq("id", set.id);
                    })
                    .await?;
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
        Ok(sqlx::query_as::<_, SettingModel>(&format!(
            "select * from {} where setting_type=? and setting_key=? and user_id=? order by id desc",
            SettingModel::table_name(),
        ))
        .bind(SettingType::Single as i8)
        .bind(&key)
        .bind(uid)
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
