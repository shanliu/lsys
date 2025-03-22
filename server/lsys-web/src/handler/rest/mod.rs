//rest接口模块
pub mod app;
pub mod auth;
#[cfg(feature = "barcode")]
pub mod barcode;
pub mod mailer;
pub mod oauth;
pub mod rbac;
pub mod smser;
