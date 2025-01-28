use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(db_type = "MySql", table_name = "setting")]
pub struct SettingModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 名称，显示用
    #[sqlx(default)]
    pub name: String,

    /// 类型
    #[sqlx(default)]
    pub setting_type: i8,

    /// 句柄
    #[sqlx(default)]
    pub setting_key: String,

    /// 数据
    #[sqlx(default)]
    pub setting_data: String,

    /// 用户ID 0系统
    #[sqlx(default)]
    pub user_id: u64,

    /// 0 禁用 1 启用
    #[sqlx(default)]
    pub status: i8,

    /// 最后修改uid
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后修改时间
    #[sqlx(default)]
    pub change_time: u64,
}
