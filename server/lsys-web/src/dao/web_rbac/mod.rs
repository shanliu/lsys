pub mod access;
#[macro_use]
mod common;
pub mod res_op;
pub mod res_tpl;

pub use common::*;
use std::sync::Arc;

use lsys_rbac::dao::RbacDao;
pub struct WebRbac {
    pub rbac_dao: Arc<RbacDao>,
}

impl WebRbac {
    pub fn new(rbac_dao: Arc<RbacDao>) -> Self {
        Self { rbac_dao }
    }
}
