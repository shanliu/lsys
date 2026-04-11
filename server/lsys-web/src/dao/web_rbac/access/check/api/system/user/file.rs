use crate::dao::{CheckResTpl, RbacCheckAccess, RbacCheckResTpl};
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

/// 用户文件上传权限
pub struct CheckUserFileUpload {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserFileUpload {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-file",
                        vec![AccessCheckOp::new("file-upload", false)],
                    )],
                    &[AccessCheckRes::system(
                        "global-file",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "file-upload",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserFileUpload {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-file",
            ops: vec!["file-upload"],
        }]
    }
}

/// 用户文件查看权限
pub struct CheckUserFileView {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserFileView {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-file",
                        vec![AccessCheckOp::new("file-view", false)],
                    )],
                    &[AccessCheckRes::system(
                        "global-file",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "file-view",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserFileView {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-file",
            ops: vec!["file-view"],
        }]
    }
}

/// 用户文件删除权限
pub struct CheckUserFileDelete {
    pub res_user_id: u64,
}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckUserFileDelete {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .list_check(
                check_env,
                &[
                    &[AccessCheckRes::system_empty_data(
                        "global-file",
                        vec![AccessCheckOp::new("file-delete", false)],
                    )],
                    &[AccessCheckRes::system(
                        "global-file",
                        &self.res_user_id.to_string(),
                        vec![AccessCheckOp::new(
                            "file-delete",
                            self.res_user_id != check_env.user_id,
                        )],
                    )],
                ],
            )
            .await
    }
}

impl RbacCheckResTpl for CheckUserFileDelete {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: true,
            data: false,
            key: "global-file",
            ops: vec!["file-delete"],
        }]
    }
}
