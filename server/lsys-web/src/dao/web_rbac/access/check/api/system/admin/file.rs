use lsys_rbac::dao::{AccessCheckEnv, AccessCheckOp, AccessCheckRes, RbacAccess, RbacResult};

use crate::dao::{
    CheckResTpl, RbacCheckAccess, RbacCheckAccessDepend, RbacCheckResTpl,
    access::api::system::admin::CheckAdminBase,
};

/// 管理员文件管理权限
pub struct CheckAdminFileManage {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminFileManage {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("file-manage", true)],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminBase {})]
    }
}

impl RbacCheckResTpl for CheckAdminFileManage {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-system",
            ops: vec!["file-manage"],
        }]
    }
}

/// 管理员导出任务管理权限（继承文件管理权限）
pub struct CheckAdminExportTaskManage {}
#[async_trait::async_trait]
impl RbacCheckAccess for CheckAdminExportTaskManage {
    async fn check(&self, access: &RbacAccess, check_env: &AccessCheckEnv<'_>) -> RbacResult<()> {
        access
            .check(
                check_env,
                &[AccessCheckRes::system_empty_data(
                    "global-system",
                    vec![AccessCheckOp::new("export-task-manage", true)],
                )],
            )
            .await
    }
    fn depends(&self) -> Vec<Box<RbacCheckAccessDepend>> {
        vec![Box::new(CheckAdminFileManage {})]
    }
}

impl RbacCheckResTpl for CheckAdminExportTaskManage {
    fn tpl_data() -> Vec<CheckResTpl> {
        vec![CheckResTpl {
            user: false,
            data: false,
            key: "global-system",
            ops: vec!["export-task-manage"],
        }]
    }
}
