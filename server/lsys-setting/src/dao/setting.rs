use std::error::Error;
use std::fmt::{Display, Formatter};
use std::ops::Deref;

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

pub trait SettingKey {
    fn key<'t>() -> &'t str;
}
pub trait SettingEncode: SettingKey {
    fn encode(&self) -> String;
}
pub trait SettingDecode: Sized + SettingKey {
    fn decode(data: &str) -> SettingResult<Self>;
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

impl<T: SettingDecode> Deref for SettingData<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
