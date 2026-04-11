// 统一导出 Trait 定义

use std::path::PathBuf;

use crate::dao::result::FileManagerError;
use crate::model::ExportTaskModel;

/// 导出任务 Trait
///
/// 各业务模块实现此 trait，通过 `register(export_type, exporter)` 注册到 `ExportTask`。
/// 同一个实现可以注册到不同的 `export_type` 下，自由组合。
///
/// 实现方负责：
/// - 从 `record` 和 `params` 中获取所需数据
/// - 内部权限校验（如需要）
/// - 写入临时文件并返回路径
///
/// 注意：权限检查建议在调用 `ExportTask::submit()` 之前在 Web 层完成，
/// 这样可以避免本模块依赖 Web 层的类型。
///
/// 泛型参数 `E` 是错误类型，必须能转换为 `FileManagerError`，由 ExportTask 在创建时指定。
pub trait Exporter<E>: Send + Sync
where
    E: Into<FileManagerError> + Send,
{
    /// 执行导出
    ///
    /// - `record`：当前导出任务的完整记录（包含 user_id、app_id、export_type 等）
    /// - `params`：从 `export_params` 解析后的 JSON
    ///
    /// 成功返回本地文件路径（管理器会 Move 到 lsys-file 存储）
    /// 失败返回错误类型 E（必须能转换为 FileManagerError）
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<PathBuf, E>> + Send + 'a>>;
}

/// 适配器：将 Exporter<E> 转换为 Exporter<FileManagerError>
pub(crate) struct ExporterAdapter<E>
where
    E: Into<FileManagerError> + Send,
{
    pub(crate) inner: Box<dyn Exporter<E>>,
}

impl<E> Exporter<FileManagerError> for ExporterAdapter<E>
where
    E: Into<FileManagerError> + Send + 'static,
{
    fn export<'a>(
        &'a self,
        record: ExportTaskModel,
        params: serde_json::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<PathBuf, FileManagerError>> + Send + 'a>,
    > {
        Box::pin(async move {
            self.inner
                .export(record, params)
                .await
                .map_err(|e| e.into())
        })
    }
}
