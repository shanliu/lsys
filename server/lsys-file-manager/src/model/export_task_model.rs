use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// 数据导出任务
///
/// 文件关联方式：任务完成后调用 FileDao 写文件，并打 TAG `export_{id}`，
/// 本表不直接存 file_id，通过 TAG 反查。
///
/// export_type 为多语言 key，前端通过 i18n 渲染类型名称。
#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_export_task")]
pub struct ExportTaskModel {
    /// 自增主键
    #[sqlx(default)]
    pub id: u64,

    /// 应用 ID，0=系统，所有操作以 app_id 为维度
    #[sqlx(default)]
    pub app_id: u64,

    /// 应用关联用户 ID，仅冗余 app 维度的用户信息，不做过滤
    #[sqlx(default)]
    pub app_user_id: u64,

    /// 用户 ID（必须，系统时为 0，用户端为当前登录用户 ID）
    #[sqlx(default)]
    pub user_id: u64,

    /// 创建导出的用户 ID（用户端与 user_id 相同，系统端为实际操作的管理员 ID）
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 导出类型标识，对应多语言 key（如 collector_record / user_list 等）
    #[sqlx(default)]
    pub export_type: String,

    /// 导出参数 JSON（过滤条件/权限由各 Exporter 实现层校验）
    #[sqlx(default)]
    pub export_params: String,

    /// 状态: 1=Pending 2=Running 3=Success 4=Failed 5=Deleted
    #[sqlx(default)]
    pub status: i8,

    /// 失败时的错误信息
    #[sqlx(default)]
    pub error_message: String,

    /// 提交并开始时间（提交即执行，无需拆分）
    #[sqlx(default)]
    pub add_time: u64,

    /// 最后状态变更时间（完成/失败/删除均更新），0=未变更
    #[sqlx(default)]
    pub change_time: u64,

    /// 请求 ID（用于追踪和关联请求）
    #[sqlx(default)]
    pub request_id: String,
}
