macro_rules! access_op {
    ($([$op:expr,$must_authorize:expr]),+$(,)*) => {
        vec![
            $(
                lsys_rbac::dao::rbac::AccessResOp {
                    op: $op.to_string(),
                    must_authorize: $must_authorize,
                }
            ),+
        ]
    };
    ($op:expr,$must_authorize:expr) => {
        lsys_rbac::dao::rbac::AccessResOp {
            op: $op.to_string(),
            must_authorize: $must_authorize,
        }
    };
}

macro_rules! access_res {
    ($([$name:expr,$ops:expr]),+$(,)*) => {
        vec![
            $(
                lsys_rbac::dao::rbac::AccessRes {
                    ops: $ops,
                    res: $name.to_string(),
                    user_id: 0,
                }
            ),+
        ]
    };
    ($([$name:expr,$user_id:expr,$ops:expr]),+$(,)*) => {
        vec![
            $(
            lsys_rbac::dao::rbac::AccessRes {
            ops: $ops,
            res: $name.to_string(),
            user_id: $user_id,
            }
            ),+
        ]
    };
    ($name:expr,$ops:expr) => {
        lsys_rbac::dao::rbac::AccessRes {
            ops:$ops,
            res: $name.to_string(),
            user_id: 0,
        }
    };
    ($name:expr,$user_id:expr,$ops:expr) => {
       lsys_rbac::dao::rbac::AccessRes {
            ops:$ops,
            res: $name.to_string(),
            user_id: $user_id,
        }
    };
}

#[macro_use]
mod res_data;
pub use res_data::*;
