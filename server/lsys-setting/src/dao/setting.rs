use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

use serde::{Deserialize, Serialize};

use crate::model::SettingModel;

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
