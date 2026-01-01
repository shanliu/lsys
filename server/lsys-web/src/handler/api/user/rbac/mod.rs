mod app;
mod system;

pub use app::*;
pub use system::*;
mod mapping;
pub use mapping::*;

//资源 角色 ... 用户

//资源:用户资源user_id>0 系统资源user_id=0
//角色:用户角色user_id>0 角色资源user_id=0
//用户:内部用户app_id=0 外部用户app_id>0

//user_id=0  为系统
//  资源&角色 为系统(后台增删改查)
//  访问用户 限定 内部用户 app_id=0

//系统内用户资源控制
//1 验证,针对访问者
//2 管理,针对所有者
//user_id>0 建立多个APP.APP开通外部用户,外部用户登录
//  app_id =0
//      资源&角色 为内置用户(后台增删改查)
//      访问用户 内部用户 app_id =0
//      访问用户 外部用户 app_id >0[多个,app_user_id=user_id]
//  app_id >0
// 1 app_id 为0 时, user_id:不为0 时 直接操作系统
// app_id:1 1u1
// app_id:2 2u1 ->访问U[2u*]
// 2 app_id 不为0 时, 为外部用户
