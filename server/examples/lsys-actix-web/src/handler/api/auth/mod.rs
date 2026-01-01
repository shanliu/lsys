//公开接口

mod login;
mod oauth;
mod password;
mod perm;
mod register;
pub use login::*;
pub use oauth::*;
pub use password::*;
pub use perm::*;
pub use register::*;
