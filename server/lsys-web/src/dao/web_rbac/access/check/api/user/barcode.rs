use crate::dao::{
    CheckRelationData, CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl,
};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};

pub struct CheckBarCodeView {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckBarCodeView {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()> {
        access
            .check(
                check_env,
                &relation.to_session_role(),
                &[AccessCheckRes::system_empty_data(
                    "global-barcode",
                    vec!["view"],
                )],
            )
            .await
    }
}
impl RbacCheckResTpl for CheckBarCodeView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-barcode",
            ops: vec!["view"],
        }]
    }
}

pub struct CheckBarCodeEdit {
    pub app_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckBarCodeEdit {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()> {
        access
            .check(
                check_env,
                &relation.to_session_role(),
                &[AccessCheckRes::system_empty_data(
                    "global-barcode",
                    vec!["edit"],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckBarCodeView {})]
    }
}
impl RbacCheckResTpl for CheckBarCodeEdit {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            key: "global-barcode",
            ops: vec!["edit"],
        }]
    }
}
