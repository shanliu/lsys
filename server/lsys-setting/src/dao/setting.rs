use std::collections::HashMap;

use std::ops::Deref;

use lsys_logger::dao::ChangeLogData;
use serde::{Deserialize, Serialize};

use crate::model::{SettingModel, SettingStatus, SettingType};

use sqlx::{MySql, Pool};
use std::sync::Arc;

use super::{MultipleSetting, SettingError, SettingResult, SingleSetting};

use lsys_core::app_core::AppCoreError;
use lsys_core::cache::LocalCacheConfig;
use lsys_core::db::SqlQuote;
use lsys_core::remote_notify::RemoteNotify;
use lsys_logger::dao::ChangeLoggerDao;

pub struct SettingConfig {
    pub single_cache: LocalCacheConfig,
    pub multiple_cache: LocalCacheConfig,
}

impl SettingConfig {
    pub fn new(use_cache: bool) -> Self {
        Self {
            single_cache: LocalCacheConfig::new(
                "setting-single",
                if use_cache { None } else { Some(0) },
                None,
            ),
            multiple_cache: LocalCacheConfig::new(
                "setting-multiple",
                if use_cache { None } else { Some(0) },
                None,
            ),
        }
    }
}

pub struct SettingDao {
    db: Pool<MySql>,
    pub single: Arc<SingleSetting>,
    pub multiple: Arc<MultipleSetting>,
}

impl SettingDao {
    pub async fn new(
        // app_core: Arc<AppCore>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: SettingConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Result<Self, AppCoreError> {
        Ok(Self {
            single: Arc::from(SingleSetting::new(
                db.clone(),
                // fluents_message.clone(),
                remote_notify.clone(),
                config.single_cache,
                logger.clone(),
            )),
            multiple: Arc::from(MultipleSetting::new(
                db.clone(),
                // fluents_message,
                remote_notify.clone(),
                config.multiple_cache,
                logger,
            )),
            db,
        })
    }
    pub async fn find_by_id(&self, id: &u64) -> SettingResult<SettingModel> {
        Ok(lsys_core::db::utils::fetch_one::<SettingModel>(
            &self.db,
            lsys_core::sql_format!("id={id} and status = {status}", id = id, status = SettingStatus::Enable),
        ).await?)
    }
    pub async fn find_by_ids(&self, ids: &[u64]) -> SettingResult<HashMap<u64, SettingModel>> {
        Ok(lsys_core::db::utils::fetch_map::<SettingModel, _, _>(
            &self.db,
            lsys_core::sql_format!("id in ({ids}) and  status = {status}", ids = ids, status = SettingStatus::Enable),
            |v| v.id,
        ).await?)
    }

    pub fn log_types() -> Vec<&'static str> {
        use lsys_logger::dao::ChangeLogData;
        vec![SettingLog::log_type()]
    }
}

pub trait SettingKey {
    fn key<'t>() -> &'t str;
}
pub trait SettingEncode: SettingKey {
    fn encode(&self) -> String;
}
pub trait SettingDecode: Sized + SettingKey {
    fn decode(data: &str) -> SettingResult<Self>;
}

//JSON方式存储配置数据
pub trait SettingJson<'t>: SettingDecode + Deserialize<'t> + SettingEncode + Serialize {
    fn decode(data: &'t str) -> SettingResult<Self> {
        serde_json::from_slice::<Self>(data.as_bytes()).map_err(SettingError::SerdeJson)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

#[derive(Clone)]
pub struct SettingData<T: SettingDecode> {
    model: SettingModel,
    data: T,
}
impl<T: SettingDecode> TryFrom<SettingModel> for SettingData<T> {
    type Error = SettingError;
    fn try_from(model: SettingModel) -> Result<Self, Self::Error> {
        let data = T::decode(&model.setting_data)?;
        Ok(Self::new(data, model))
    }
}
impl<T: SettingDecode> SettingData<T> {
    pub fn new(data: T, model: SettingModel) -> Self {
        Self { model, data }
    }
    pub fn model(&self) -> &SettingModel {
        &self.model
    }
}
impl<T: SettingDecode + Default> Default for SettingData<T> {
    fn default() -> Self {
        Self {
            model: SettingModel::default(),
            data: T::default(),
        }
    }
}

impl<T: SettingDecode> Deref for SettingData<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

#[derive(Serialize)]
pub(crate) struct SettingLog<'t> {
    pub action: &'t str,
    pub user_id: u64,
    pub setting_key: &'t str,
    pub setting_type: SettingType,
    pub name: &'t str,
    pub setting_data: &'t str,
}

impl ChangeLogData for SettingLog<'_> {
    fn log_type() -> &'static str {
        "setting"
    }
    fn message(&self) -> String {
        format!("{}:{}[{}]", self.action, self.name, self.setting_key)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
