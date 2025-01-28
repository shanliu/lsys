//rbac权限模块扩充
pub mod access;
mod common;
pub use common::*;
use std::sync::Arc;

use lsys_rbac::dao::RbacDao;

pub const APP_FEATURE_RBAC: &str = "rbac";
pub struct WebRbac {
    pub rbac_dao: Arc<RbacDao>,
}

impl WebRbac {
    pub fn new(rbac_dao: Arc<RbacDao>) -> Self {
        Self { rbac_dao }
    }
}
