//所有接口公共部分

#[cfg(feature = "area")]
#[macro_use]
pub mod area;

pub mod app;
pub mod rbac;
pub mod sender;
pub mod setting;
pub mod user;

#[cfg(feature = "docs")]
pub mod docs;

#[cfg(feature = "barcode")]
pub mod barcode;
