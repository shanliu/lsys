use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserAddressEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAddressEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-user",
                    vec![AccessCheckOp::new(
                        "address-edit",
                        self.res_user_id != check_env.user_id,
                    )],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserAddressEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-user",
            ops: vec!["address-base"],
        }]
    }
}
