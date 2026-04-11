use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 采集脚本配置
#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_collector_script")]
pub struct CollectorScriptModel {
    /// 自增主键
    #[sqlx(default)]
    pub id: u64,

    /// 创建用户 ID
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 应用关联用户 ID，仅冗余，不做过滤
    #[sqlx(default)]
    pub app_user_id: u64,

    /// 应用 ID，0=系统
    #[sqlx(default)]
    pub app_id: u64,

    /// 脚本名称（唯一标识，用作文件 TAG）
    #[sqlx(default)]
    pub name: String,

    /// JS 脚本代码
    #[sqlx(default)]
    pub script_code: String,

    /// 脚本代码 MD5
    #[sqlx(default)]
    pub script_md5: String,

    /// 执行超时秒数，默认 30
    #[sqlx(default)]
    pub timeout_secs: u32,

    /// 内存限制（字节），默认 64MB
    #[sqlx(default)]
    pub memory_limit: u64,

    /// 状态: 1=启用, 2=禁用, 3=已删除
    #[sqlx(default)]
    pub status: i8,

    /// 创建时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 修改时间
    #[sqlx(default)]
    pub change_time: u64,
}

/// 采集执行记录
#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_collector_record")]
pub struct CollectorRecordModel {
    /// 自增主键
    #[sqlx(default)]
    pub id: u64,

    /// 请求 ID（来自 RequestEnv.request_id，或自动生成）
    #[sqlx(default)]
    pub request_id: String,

    /// 脚本 ID
    #[sqlx(default)]
    pub script_id: u64,

    /// 触发用户 ID
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 应用 ID
    #[sqlx(default)]
    pub app_id: u64,

    /// JsTaskRunner 分配的 task_id
    #[sqlx(default)]
    pub task_id: u64,

    /// 执行参数（JSON 字符串）
    #[sqlx(default)]
    pub exec_params: String,

    /// 状态: 1=Pending, 2=Running, 3=Success, 4=Failed, 5=Timeout
    #[sqlx(default)]
    pub status: i8,

    /// 执行耗时（毫秒）
    #[sqlx(default)]
    pub elapsed_ms: u64,

    /// 错误信息（失败/超时时填入）
    #[sqlx(default)]
    pub error_message: String,

    /// 提交时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 开始执行时间
    #[sqlx(default)]
    pub start_time: u64,

    /// 完成时间
    #[sqlx(default)]
    pub finish_time: u64,
}

/// 采集日志（JS 脚本日志 + 系统关键操作日志）
#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_collector_log")]
pub struct CollectorLogModel {
    /// 自增主键
    #[sqlx(default)]
    pub id: u64,

    /// 请求 ID（关联 CollectorRecordModel.request_id）
    #[sqlx(default)]
    pub request_id: String,

    /// 脚本 ID
    #[sqlx(default)]
    pub script_id: u64,

    /// 触发用户 ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 应用 ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 日志等级: 0=Trace, 1=Debug, 2=Info, 3=Warn, 4=Error, 10=System
    #[sqlx(default)]
    pub level: u8,

    /// 日志消息
    #[sqlx(default)]
    pub message: String,

    /// 写入时间
    #[sqlx(default)]
    pub add_time: u64,
}
