use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckUserBarCodeView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserBarCodeView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-user",
                        vec![AccessCheckOp::new(
                            "view-barcode",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                    &[AccessCheckRes::system(
                        "global-user",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "view-barcode",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckUserBarCodeView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-user",
                ops: vec!["view-barcode"],
            },
            CheckResTpl {
                user: false,
                data: true,
                key: "global-user",
                ops: vec!["view-barcode"],
            },
        ]
    }
}

pub struct CheckUserBarCodeEdit {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserBarCodeEdit {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-user",
                        vec![AccessCheckOp::new(
                            "edit-barcode",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                    &[AccessCheckRes::system(
                        "global-user",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "edit-barcode",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
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
        vec![
            CheckResTpl {
                user: false,
                data: false,
                key: "global-user",
                ops: vec!["edit-barcode"],
            },
            CheckResTpl {
                user: false,
                data: true,
                key: "global-user",
                ops: vec!["edit-barcode"],
            },
        ]
    }
}
