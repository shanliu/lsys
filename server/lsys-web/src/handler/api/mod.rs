// 页面接口
pub mod login;
pub use super::common::app;
#[cfg(feature = "area")]
pub use super::common::area;
#[cfg(feature = "barcode")]
pub use super::common::barcode;
pub use super::common::docs;
pub use super::common::rbac;
pub use super::common::sender;
pub use super::common::setting;
pub use super::common::user;
