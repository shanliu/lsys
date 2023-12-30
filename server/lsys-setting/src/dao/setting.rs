use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use lsys_logger::dao::ChangeLogData;
use serde::{Deserialize, Serialize};

use crate::model::{SettingModel, SettingStatus, SettingType};

use sqlx::{MySql, Pool};
use std::sync::Arc;

use super::{MultipleSetting, SingleSetting};

use lsys_core::{AppCore, AppCoreError, RemoteNotify};
use lsys_logger::dao::ChangeLogger;
pub struct Setting {
    db: Pool<MySql>,
    pub single: Arc<SingleSetting>,
    pub multiple: Arc<MultipleSetting>,
}

impl Setting {
    pub async fn new(
        app_core: Arc<AppCore>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLogger>,
    ) -> Result<Self, AppCoreError> {
        let app_locale_dir = app_core.app_dir.join("locale/lsys-setting");
        let fluents_message = Arc::new(if app_locale_dir.exists() {
            app_core.create_fluent(app_locale_dir).await?
        } else {
            let cargo_dir = env!("CARGO_MANIFEST_DIR");
            app_core
                .create_fluent(cargo_dir.to_owned() + "/locale")
                .await?
        });
        Ok(Self {
            single: Arc::from(SingleSetting::new(
                db.clone(),
                fluents_message.clone(),
                remote_notify.clone(),
                logger.clone(),
            )),
            multiple: Arc::from(MultipleSetting::new(
                db.clone(),
                fluents_message,
                remote_notify,
                logger,
            )),
            db,
        })
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        SettingModel,
        SettingResult<SettingModel>,
        id,
        "id={id} and status = {status}",
        status = SettingStatus::Enable
    );
    lsys_core::impl_dao_fetch_map_by_vec!(
        db,
        find_by_ids,
        u64,
        SettingModel,
        SettingResult<HashMap<u64, SettingModel>>,
        id,
        ids,
        "id in ({ids}) and  status = {status}",
        status = SettingStatus::Enable
    );
}

#[derive(Debug)]
pub enum SettingError {
    Sqlx(sqlx::Error),
    System(String),
}
impl Display for SettingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Error for SettingError {}

impl From<sqlx::Error> for SettingError {
    fn from(err: sqlx::Error) -> Self {
        SettingError::Sqlx(err)
    }
}

pub type SettingResult<T> = Result<T, SettingError>;

pub trait NotFoundDefault {
    fn notfound_default(self) -> Self;
}

impl<T: Default> NotFoundDefault for SettingResult<T> {
    fn notfound_default(self) -> Self {
        match self {
            Ok(s) => Ok(s),
            Err(SettingError::Sqlx(sqlx::Error::RowNotFound)) => Ok(T::default()),
            Err(e) => Err(e),
        }
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
        serde_json::from_slice::<Self>(data.as_bytes())
            .map_err(|e| SettingError::System(e.to_string()))
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
pub(crate) struct SettingLog {
    pub action: &'static str,
    pub setting_key: String,
    pub setting_type: SettingType,
    pub name: String,
    pub setting_data: String,
}

impl ChangeLogData for SettingLog {
    fn log_type<'t>() -> &'t str {
        "setting"
    }
    fn message(&self) -> String {
        format!("{}:{}[{}]", self.action, self.name, self.setting_key)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
