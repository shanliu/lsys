// 统一导出 Trait 定义

use std::path::PathBuf;

use crate::dao::WebResult;

use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::WebError;
use crate::model::ExportTaskModel;

/// 导出任务 Trait
///
/// 各业务模块实现此 trait，通过 `register(export_type, exporter)` 注册到 `WebExportTask`。
/// 同一个实现可以注册到不同的 `export_type` 下，自由组合。
///
/// 实现方负责：
/// - 从 `record` 和 `params` 中获取所需数据
/// - 内部权限校验
/// - 写入临时文件并返回路径
pub trait Exporter: Send + Sync {
    /// 权限校验
    fn check<'a>(
        &'a self,
        _check_env: &'a RbacAccessCheckEnv<'_>,
        _app_id: u64,
        _app_user_id: u64,
        _user_id: u64,
        _export_type: &'a str,
        _params: &'a serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = WebResult<()>> + Send + 'a>> {
        Box::pin(async { Ok(()) })
    }
    /// 执行导出
    ///
    /// - `record`：当前导出任务的完整记录（包含 user_id、app_id、export_type 等）
    /// - `params`：从 `export_params` 解析后的 JSON
    ///
    /// 成功返回本地文件路径（管理器会 Move 到 lsys-files 存储）
    /// 失败返回 `WebError`
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PathBuf, WebError>> + Send + 'a>>;
}
