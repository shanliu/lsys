use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::db::OffsetPageParam;
use lsys_core::db::{
    utils::FetchField, Insert, OptionTxExecutor, QueryBuilderExt, TableMeta,
    Update,
};
use lsys_core::fluent_message;
use lsys_core::fluents::IntoFluentMessage;
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{now_time, RequestEnv};
use lsys_core::valid_param::{
    ValidError, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};
use lsys_core::valid_key;
use lsys_logger::dao::ChangeLoggerDao;
use sqlx::{MySql, Pool, QueryBuilder, Transaction};
use std::sync::Arc;
use tracing::log::warn;

use super::{SettingData, SettingDecode, SettingEncode, SettingError, SettingLog, SettingResult};
use crate::model::{SettingModel, SettingStatus, SettingType};

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
}
pub struct MultipleSettingData<'t, T: SettingEncode> {
    pub name: &'t str,
    pub data: &'t T,
}
impl MultipleSetting {
    async fn add_param_valid(&self, key: &str, name: &str, data: &str) -> SettingResult<()> {
        let fetch_field = FetchField::new(&self.db);
        let setting_key_max =
           fetch_field.string_max::<SettingModel>( &SettingModel::SETTING_KEY)
                .await
                .len_or(32);
        let name_max = fetch_field.string_max::<SettingModel>( &SettingModel::NAME)
            .await
            .len_or(32);
        let setting_data_max =
           fetch_field.string_max::<SettingModel>( &SettingModel::SETTING_DATA)
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
    pub async fn add<T: SettingEncode>(
        &self,
        user_id: Option<u64>,
        param: &MultipleSettingData<'_, T>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let change_user_id = change_user_id.to_owned();
        let setting_type = SettingType::Multiple as i8;
        let status = SettingStatus::Enable as i8;
        let edata = param.data.encode();
        let key = T::key().to_string();
        let time = now_time().unwrap_or_default();
        let uid = user_id.unwrap_or_default();
        let name = param.name.to_owned();

        self.add_param_valid(&key, &name, &edata).await?;

        let find_res = sqlx::query_scalar::<_, u64>(&format!(
            "select id from {} where setting_type=? and user_id=? and setting_key=? and status=? and name=?",
            SettingModel::table_name(),
        ))
        .bind(setting_type)
        .bind(uid)
        .bind(&key)
        .bind(status)
        .bind(&name)
        .fetch_one(&self.db)
        .await;
        match find_res {
            Ok(id) => {
                return Err(SettingError::Vaild(ValidError::message(
                    valid_key!("setting_name"),
                    fluent_message!("setting-name-exits",{
                        "id":id,
                        "name":name
                    }),
                )))
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => return Err(err)?,
        }

        let dat = Insert::<_, SettingModel>::new()
            .set(SettingModel::NAME, &name)
            .set(SettingModel::SETTING_TYPE, setting_type)
            .set(SettingModel::SETTING_KEY, &key)
            .set(SettingModel::SETTING_DATA, &edata)
            .set(SettingModel::USER_ID, uid)
            .set(SettingModel::STATUS, status)
            .set(SettingModel::CHANGE_USER_ID, change_user_id)
            .set(SettingModel::CHANGE_TIME, time)
            .execute(OptionTxExecutor::new(transaction, &self.db))
            .await?;
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
    async fn edit_param_valid(
        &self,
        id: u64,
        key: &str,
        name: &str,
        data: &str,
    ) -> SettingResult<()> {
        let fetch_field = FetchField::new(&self.db);
        let setting_key_max =
           fetch_field.string_max::<SettingModel>( &SettingModel::SETTING_KEY)
                .await
                .len_or(32);
        let name_max = fetch_field.string_max::<SettingModel>( &SettingModel::NAME)
            .await
            .len_or(32);
        let setting_data_max =
           fetch_field.string_max::<SettingModel>( &SettingModel::SETTING_DATA)
                .await
                .len_or(60000);

        ValidParam::default()
            .add(
                valid_key!("setting_id"),
                &id,
                &ValidParamCheck::default().add_rule(ValidNumber::id()),
            )
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
    pub async fn edit<T: SettingEncode>(
        &self,
        user_id: Option<u64>,
        id: u64,
        param: &MultipleSettingData<'_, T>,
        change_user_id: u64,
        transaction: Option<&mut Transaction<'_, sqlx::MySql>>,
        env_data: Option<&RequestEnv>,
    ) -> SettingResult<u64> {
        let id = id.to_owned();
        let change_user_id = change_user_id.to_owned();
        let name = param.name.to_owned();
        let edata = param.data.encode();
        let key = T::key().to_string();
        let uid = user_id.unwrap_or_default();

        self.edit_param_valid(id, &key, &name, &edata).await?;

        let find_res = sqlx::query_scalar::<_, u64>(&format!(
            "select id from {} where setting_type=? and user_id=? and setting_key=? and status=? and name=? and id!=?",
            SettingModel::table_name(),
        ))
        .bind(SettingType::Multiple as i8)
        .bind(uid)
        .bind(&key)
        .bind(SettingStatus::Enable as i8)
        .bind(&name)
        .bind(id)
        .fetch_one(&self.db)
        .await;
        match find_res {
            Ok(id) => {
                return Err(SettingError::Vaild(ValidError::message(
                    valid_key!("setting_name"),
                    fluent_message!("setting-name-exits",{
                        "id":id,
                        "name":name
                    }),
                )))
            }
            Err(sqlx::Error::RowNotFound) => {}
            Err(err) => return Err(err)?,
        }

        let time = now_time().unwrap_or_default();

        let cu = Update::<_, SettingModel>::new()
            .set(SettingModel::SETTING_DATA, &edata)
            .set(SettingModel::NAME, &name)
            .set(SettingModel::CHANGE_USER_ID, change_user_id)
            .set(SettingModel::CHANGE_TIME, time)
            .execute(
                OptionTxExecutor::new(transaction, &self.db),
                |qb| {
                    qb.push_where().field_eq("id", id);
                    qb.push_and().field_eq("setting_type", SettingType::Multiple as i8);
                    qb.push_and().field_eq("setting_key", key.to_owned());
                    qb.push_and().field_eq("user_id", uid);
                },
            )
            .await?;

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

        let uid = user_id.unwrap_or_default();

        let data = sqlx::query_as::<_, SettingModel>(&format!(
            "select * from {} where id=? and setting_type=? and setting_key=? and  user_id=?",
            SettingModel::table_name(),
        ))
        .bind(id)
        .bind(SettingType::Multiple as i8)
        .bind(&key)
        .bind(uid)
        .fetch_one(&self.db)
        .await;

        match data {
            Ok(item) => {
                if SettingStatus::Delete.eq(item.status) {
                    return Ok(0);
                }
                let cu = Update::<_, SettingModel>::new()
                    .set(SettingModel::STATUS, status)
                    .set(SettingModel::CHANGE_USER_ID, change_user_id)
                    .set(SettingModel::CHANGE_TIME, time)
                    .execute(
                        OptionTxExecutor::new(transaction, &self.db),
                        |qb| {
                            qb.push_where().field_eq("id", item.id);
                        },
                    )
                    .await?;
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
        user_id: Option<u64>,
        ids: Option<&[u64]>,
        page: &OffsetPageParam,
    ) -> SettingResult<Vec<SettingData<T>>> {
        let key = T::key().to_string();
        let uid = user_id.unwrap_or_default();
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "select * from {}",
            SettingModel::table_name(),
        ));
        qb.push_where().field_eq("status", SettingStatus::Enable as i8);
        qb.push_and().field_eq("setting_type", SettingType::Multiple as i8);
        qb.push_and().field_eq("setting_key", key.to_owned());
        qb.push_and().field_eq("user_id", uid);
        if let Some(id) = ids {
            if id.is_empty() {
                return Ok(vec![]);
            }
            qb.push_and().field_in_copied(" id", id);
        }
        page.push_limit(&mut qb);

        let data = qb.build_query_as::<SettingModel>().fetch_all(&self.db).await?;

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
        let sql = format!(
            "select count(*) as total from {} where status=? and setting_type=? and setting_key=? and  user_id=?",
            SettingModel::table_name(),
        );
        let query = sqlx::query_scalar::<_, i64>(&sql)
            .bind(SettingStatus::Enable as i8)
            .bind(SettingType::Multiple as i8)
            .bind(&key)
            .bind(uid);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    pub async fn find(&self, user_id: Option<u64>, id: u64) -> SettingResult<SettingModel> {
        let id = id.to_owned();
        let uid = user_id.unwrap_or_default();

        Ok(sqlx::query_as::<_, SettingModel>(&format!(
            "select * from {} where id=? and setting_type=? and  user_id=?",
            SettingModel::table_name(),
        ))
        .bind(id)
        .bind(SettingType::Multiple as i8)
        .bind(uid)
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
