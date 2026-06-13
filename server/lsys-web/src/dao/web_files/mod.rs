//文件模块封装

mod web_collector;
mod web_export;
mod web_file;

// 重新导出各子模块的公共类型
pub use self::web_collector::WebCollector;
pub use self::web_export::{WebExport, WebExportCheckParam, WebExporterCheck, WebExportTask};
pub use self::web_file::WebFile;

// 重新导出 lsys-file-manager 的类型，供 web 层使用
pub use lsys_file_manager::ExportTask;
pub use lsys_file_manager::FileCollector as FileCollectorType;

// 重新导出 export_task 子模块
pub mod export_task_types {
    pub use lsys_file_manager::dao::export_task::exporter;
    pub use lsys_file_manager::dao::export_task::writer;
    pub use lsys_file_manager::dao::export_task::{ExportTaskFileItem, ExportTaskItem};
}
