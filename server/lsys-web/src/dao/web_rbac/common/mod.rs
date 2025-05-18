//静态权限模块扩充及权限模版实现
mod depend;
#[macro_use]
mod tpls;
pub use depend::*;
pub use tpls::*;
