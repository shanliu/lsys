use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

/// 导出任务状态
/// 导出任务状态
///
/// DDL: `status TINYINT NOT NULL DEFAULT 1 COMMENT '状态: 1=Pending 2=Running 3=Success 4=Failed 5=Deleted'`
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum ExportTaskStatus {
    /// 待执行（刚插入，等待调度）
    Pending = 1,
    /// 执行中（已被调度器取走）
    Running = 2,
    /// 成功（文件已通过 TAG 关联到 lsys-file）
    Success = 3,
    /// 失败（含超时标记失败，error_message 记录原因）
    Failed = 4,
    /// 已删除（手动操作，change_time 记录删除时间）
    Deleted = 5,
}
