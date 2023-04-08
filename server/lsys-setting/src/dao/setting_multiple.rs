use lsys_core::cache::{LocalCache, LocalCacheConfig};
use lsys_core::{now_time, FluentMessage, PageParam};
use sqlx::{MySql, Pool};
use sqlx_model::{model_option_set, sql_format, Insert, ModelTableName, Select, Update};
use std::sync::Arc;

use super::{SettingData, SettingDecode, SettingEncode, SettingResult};
use crate::model::{SettingModel, SettingModelRef, SettingStatus, SettingType};
use sqlx_model::SqlQuote;

pub struct MultipleSetting {
    db: Pool<MySql>,
    //fluent: Arc<FluentMessage>,
    pub cache: Arc<LocalCache<String, Vec<SettingModel>>>,
}

impl MultipleSetting {
    pub fn new(db: Pool<MySql>, _fluent: Arc<FluentMessage>, redis: deadpool_redis::Pool) -> Self {
        Self {
            cache: Arc::from(LocalCache::new(redis, LocalCacheConfig::new("setting"))),
            db,
            // fluent,
        }
    }
    pub async fn add<T: SettingEncode>(
        &self,
        user_id: &Option<u64>,
        name: &str,
        data: &T,
        change_user_id: &u64,
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
            last_user_id: change_user_id,
            last_change_time: time,
        });
        let dat = Insert::<sqlx::MySql, SettingModel, _>::new(new_data)
            .execute(&self.db)
            .await?;
        self.cache.clear(&format!("{}-{}", key, uid)).await;
        Ok(dat.last_insert_id())
    }
    pub async fn edit<T: SettingEncode>(
        &self,
        user_id: &Option<u64>,
        id: &u64,
        name: &str,
        data: &T,
        change_user_id: &u64,
    ) -> SettingResult<u64> {
        let id = id.to_owned();
        let change_user_id = change_user_id.to_owned();
        let name = name.to_owned();
        let edata = data.encode();
        let key = T::key().to_string();
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SettingModelRef,{
            setting_data: edata,
            name:name,
            last_user_id: change_user_id,
            last_change_time: time,
        });
        let uid = user_id.unwrap_or_default();
        let cu = Update::<sqlx::MySql, SettingModel, _>::new(change)
            .execute_by_where_call(
                "id=? and setting_type=? and setting_key=? and user_id=?",
                |res, _| {
                    res.bind(id)
                        .bind(SettingType::Multiple as i8)
                        .bind(key.clone())
                        .bind(uid)
                },
                &self.db,
            )
            .await?;
        self.cache.clear(&format!("{}-{}", key, uid)).await;
        Ok(cu.rows_affected())
    }
    pub async fn del<T: SettingEncode>(
        &self,
        user_id: &Option<u64>,
        id: &u64,
        change_user_id: &u64,
    ) -> SettingResult<u64> {
        let change_user_id = change_user_id.to_owned();
        let id = id.to_owned();
        let key = T::key().to_string();
        let status = SettingStatus::Delete as i8;
        let time = now_time().unwrap_or_default();
        let change = sqlx_model::model_option_set!(SettingModelRef,{
            status: status,
            last_user_id: change_user_id,
            last_change_time: time,
        });
        let uid = user_id.unwrap_or_default();
        let cu = Update::<sqlx::MySql, SettingModel, _>::new(change)
            .execute_by_where_call(
                "id=? and setting_type=? and setting_key=? and  user_id=?",
                |res, _| {
                    res.bind(id)
                        .bind(SettingType::Multiple as i8)
                        .bind(key.clone())
                        .bind(uid)
                },
                &self.db,
            )
            .await?;
        self.cache.clear(&format!("{}-{}", key, uid)).await;
        Ok(cu.rows_affected())
    }
    pub async fn list_data<T: SettingDecode>(
        &self,
        user_id: &Option<u64>,
        ids: &Option<Vec<u64>>,
        page: &Option<PageParam>,
    ) -> SettingResult<Vec<SettingData<T>>> {
        let key = T::key().to_string();
        let uid = user_id.unwrap_or_default();
        let mut sql = sql_format!(
            "setting_type={} and setting_key={} and  user_id={}",
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

        let data = Select::type_new::<SettingModel>()
            .fetch_all_by_where::<SettingModel, _>(&sqlx_model::WhereOption::Where(sql), &self.db)
            .await?;
        let mut out = Vec::with_capacity(data.len());
        for model in data {
            let dat = T::decode(&model.setting_data)?;
            out.push(SettingData::new(dat, model));
        }
        Ok(out)
    }
    pub async fn list_count<T: SettingDecode>(&self, user_id: &Option<u64>) -> SettingResult<i64> {
        let key = T::key().to_string();
        let uid = user_id.unwrap_or_default();
        let sql = sql_format!(
            "select count(*) as total from {} where setting_type={} and setting_key={} and  user_id={}",
            SettingModel::table_name(),
            SettingType::Multiple as i8,
            key,
            uid
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }
    pub async fn load<T: SettingDecode>(
        &self,
        user_id: &Option<u64>,
        id: &u64,
    ) -> SettingResult<SettingData<T>> {
        let id = id.to_owned();
        let key = T::key().to_string();
        let uid = user_id.unwrap_or_default();
        let model = Select::type_new::<SettingModel>()
            .fetch_one_by_where_call::<SettingModel, _, _>(
                "id=? and setting_type=? and setting_key=? and user_id=?",
                |res, _| {
                    res.bind(id)
                        .bind(SettingType::Multiple as i8)
                        .bind(key.clone())
                        .bind(uid)
                },
                &self.db,
            )
            .await?;
        let data = T::decode(&model.setting_data)?;
        Ok(SettingData::new(data, model))
    }
}
