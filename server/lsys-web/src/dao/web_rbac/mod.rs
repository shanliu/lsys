mod app_feature;
mod common;
//rbac权限模块扩充
pub mod access;
pub mod res_tpl;
pub mod user;

pub use common::*;
use lsys_app::dao::AppDao;
use std::sync::Arc;

use lsys_rbac::dao::RbacDao;

pub struct WebRbac {
    pub rbac_dao: Arc<RbacDao>,
    app_dao: Arc<AppDao>,
}

impl WebRbac {
    pub fn new(rbac_dao: Arc<RbacDao>, app_dao: Arc<AppDao>) -> Self {
        Self { rbac_dao, app_dao }
    }
}
