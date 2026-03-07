use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

/// 采集脚本状态
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum CollectorScriptStatus {
    /// 启用
    Enable = 1,
    /// 禁用
    Disable = 2,
    /// 已删除
    Deleted = 3,
}

/// 采集执行记录状态
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum CollectorRecordStatus {
    /// 待执行
    Pending = 1,
    /// 执行中
    Running = 2,
    /// 成功
    Success = 3,
    /// 失败
    Failed = 4,
    /// 超时
    Timeout = 5,
}

/// 采集日志等级（与 lsys_lib_jsrun LOG_LEVEL_* 对齐）
pub const COLLECTOR_LOG_LEVEL_TRACE: u8 = 0;
pub const COLLECTOR_LOG_LEVEL_DEBUG: u8 = 1;
pub const COLLECTOR_LOG_LEVEL_INFO: u8 = 2;
pub const COLLECTOR_LOG_LEVEL_WARN: u8 = 3;
pub const COLLECTOR_LOG_LEVEL_ERROR: u8 = 4;
/// 系统操作日志（关键流程自动写入）
pub const COLLECTOR_LOG_LEVEL_SYSTEM: u8 = 10;
