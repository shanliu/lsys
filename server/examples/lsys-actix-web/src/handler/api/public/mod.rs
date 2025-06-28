//公开接口

pub mod app;
mod area;
mod captcha;
mod login;
mod oauth;
mod options;
mod password;
mod register;
mod site;
pub use area::*;
pub use captcha::*;
pub use login::*;
pub use oauth::*;
pub use options::*;
pub use password::*;
pub use register::*;
pub use site::*;
