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

#[test]
fn test_access_res_macro() {
    let a1 = access_res!(
        "user",
        access_op!(["email_add", false], ["email_view", true])
    );
    assert!(a1.res == "user");
    assert!(a1.ops[0].op == "email_add");
    assert!(!a1.ops[0].must_authorize);
    assert!(a1.ops[1].op == "email_view");
    assert!(a1.ops[1].must_authorize);

    let a1 = access_res!(["user", 1, access_op!(["email_add", false])]);
    assert!(a1[0].user_id == 1);

    let a1 = access_res!(
        ["user", access_op!(["email_add", false])],
        ["page", access_op!(["page_add", false])]
    );
    assert!(a1.len() == 2);
    assert!(a1[0].res == "user");
    assert!(a1[1].res == "page");

    let a1 = access_res!(
        ["user", 11, vec![access_op!("email_add", false)]],
        [
            "page",
            2,
            access_op!(["page_add", false], ["page_edit", false])
        ]
    );
    assert!(a1.len() == 2);
    assert!(a1[0].user_id == 11);
    assert!(a1[1].user_id == 2);
}

#[macro_use]
mod res_data;
mod app;
pub use app::*;
pub use res_data::*;
