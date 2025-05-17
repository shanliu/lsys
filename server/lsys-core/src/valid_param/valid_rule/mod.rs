// 校验码封装
mod china_id;
mod china_license_plate;
mod color;
mod contains;
mod credit_card;
mod datetime;
mod domain;
mod email;
mod git;
mod ip;
mod mobile;
mod not_empty;
mod nunber;
mod password;
mod pattern;
mod str_match;
mod strlen;
mod url;
pub use china_id::*;
pub use china_license_plate::*;
pub use color::*;
pub use contains::*;
pub use credit_card::*;
pub use datetime::*;
pub use domain::*;
pub use email::*;
pub use git::*;
pub use ip::*;
pub use mobile::*;
pub use not_empty::*;
pub use nunber::*;
pub use password::*;
pub use pattern::*;
pub use str_match::*;
pub use strlen::*;
pub use url::*;

use super::ValidRuleError;

pub trait ValidRule {
    type T;
    fn check(&self, data: &Self::T) -> Result<(), ValidRuleError>;
}
