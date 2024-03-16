use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::sqlx_model;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "barcode_output")]
pub struct BarcodeOutputModel {
    /// id
    #[sqlx(default)]
    pub id: u64,
    /// app_id
    #[sqlx(default)]
    pub app_id: u64,

    #[sqlx(default)]
    pub code_format: u8,

    #[sqlx(default)]
    pub image_format: u8,

    #[sqlx(default)]
    pub image_size: u64,

    #[sqlx(default)]
    pub image_color_front: u32,

    #[sqlx(default)]
    pub image_color_background: u32,

    #[sqlx(default)]
    pub image_background: String,

    /// 用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 下次推送时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "barcode_parse")]
pub struct BarcodeParseModel {
    /// id
    #[sqlx(default)]
    pub id: u64,
    /// app_id
    #[sqlx(default)]
    pub app_id: u64,

    #[sqlx(default)]
    pub file_path: String,

    #[sqlx(default)]
    pub file_hash: String,

    #[sqlx(default)]
    pub file_size: u64,

    #[sqlx(default)]
    pub record: String,

    /// 用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 下次推送时间
    #[sqlx(default)]
    pub create_time: u64,
}
