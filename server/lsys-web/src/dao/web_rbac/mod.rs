pub mod access;
#[macro_use]
mod common;
pub mod res_op;
pub mod res_tpl;

pub use common::*;
use std::sync::Arc;

use lsys_rbac::dao::RbacDao;
pub struct WebRbac {
    root_id_data: Vec<u64>,
    pub rbac_dao: Arc<RbacDao>,
}

impl WebRbac {
    pub fn new(rbac_dao: Arc<RbacDao>, root_id_data: Vec<u64>) -> Self {
        Self {
            rbac_dao,
            root_id_data,
        }
    }
    pub fn is_root(&self, user_id: u64) -> bool {
        self.root_id_data.contains(&user_id)
    }
}
