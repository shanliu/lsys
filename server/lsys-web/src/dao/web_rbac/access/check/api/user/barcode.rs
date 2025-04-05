use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserBarCodeView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserBarCodeView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-barcode",
                    vec!["view"],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserBarCodeView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-barcode",
            ops: vec!["view"],
        }]
    }
}

pub struct CheckUserBarCodeEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserBarCodeEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::user_empty_data(
                    self.res_user_id,
                    "global-barcode",
                    vec!["edit"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckUserBarCodeView {
            res_user_id: self.res_user_id,
        })]
    }
}
impl RbacCheckResTpl for CheckUserBarCodeEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-barcode",
            ops: vec!["edit"],
        }]
    }
}
