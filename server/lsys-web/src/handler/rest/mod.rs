//rest接口模块
mod auth;
mod info;
mod mailer;
mod oauth;
mod rbac;
mod smser;
pub use auth::*;
pub use info::*;
pub use mailer::*;
pub use oauth::*;
pub use rbac::*;
pub use smser::*;
#[cfg(feature = "barcode")]
mod barcode;
#[cfg(feature = "barcode")]
pub use barcode::*;
