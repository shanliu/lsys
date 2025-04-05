use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserAddressBase {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAddressBase {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::system_empty_data(
                    "global-public",
                    vec!["address-base"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserAddressBase {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-public",
            ops: vec!["address-base"],
        }]
    }
}

pub struct CheckUserAddressEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserAddressEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env, //资源访问用户
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-public",
                    vec!["address-edit"],
                )],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserAddressEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-public",
            ops: vec!["address-edit"],
        }]
    }
}
