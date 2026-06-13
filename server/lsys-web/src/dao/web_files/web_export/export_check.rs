// Web 层导出器 trait

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::result::WebResult;

/// 导出权限检查参数
///
/// 封装导出任务权限检查所需的上下文信息
#[derive(Debug, Clone)]
pub struct WebExportCheckParam<'a> {
    /// 应用 ID
    pub app_id: u64,
    /// 应用用户 ID
    pub app_user_id: u64,
    /// 用户 ID
    pub user_id: u64,
    /// 导出类型
    pub export_type: &'a str,
    /// 导出参数
    pub params: &'a serde_json::Value,
}


/// Web 层导出器 trait
///
/// 继承自 lsys-file-manager 的 Exporter trait，并添加权限检查方法
/// 使用 WebError 作为错误类型
#[async_trait::async_trait]
pub trait WebExporterCheck: Send + Sync {
    /// 权限检查
    ///
    /// 在执行导出前检查用户是否有权限执行此导出操作
    ///
    /// # 参数
    /// - `check_env`: RBAC 访问检查环境
    /// - `param`: 导出检查参数
    async fn check(
        &self,
        check_env: &RbacAccessCheckEnv<'_>,
        param: &WebExportCheckParam<'_>,
    ) -> WebResult<()>;
}
