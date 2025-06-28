mod check;
mod data_audit;
mod data_res;
mod data_user;

pub use {
    check::AccessCheckEnv, check::AccessCheckOp, check::AccessCheckRes, check::AccessSessionRole,
    check::AccessUnauthRes,
};

pub use data_audit::AuditDataParam;
pub use data_res::*;
pub use data_user::*;

//授权检查实现

use super::{op::RbacOp, res::RbacRes, role::RbacRole};
use check::AuditItem;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

pub struct RbacAccess {
    root_id_data: Vec<u64>,
    db: sqlx::Pool<sqlx::MySql>,
    role: Arc<RbacRole>,
    res: Arc<RbacRes>,
    op: Arc<RbacOp>,
    audit_sender: Option<Sender<AuditItem>>,
}

impl RbacAccess {
    pub fn new(
        root_id_data: Vec<u64>,
        db: sqlx::Pool<sqlx::MySql>,
        role: Arc<RbacRole>,
        res: Arc<RbacRes>,
        op: Arc<RbacOp>,
        delay_audit_num: usize, //审计日志延迟插入队列数量
    ) -> Self {
        Self {
            audit_sender: Self::listen_audit(db.clone(), delay_audit_num),
            root_id_data,
            db,
            role,
            res,
            op,
        }
    }
    //指定用户ID是否为ROOT用户
    pub fn is_root(&self, user_id: u64) -> bool {
        self.root_id_data.contains(&user_id)
    }
}
