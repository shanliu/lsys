use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::sqlx_model;

      


#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "barcode_create")]
pub struct BarcodeCreateModel {
    /// id
    #[sqlx(default)]
    pub id: u64,
    /// app_id
    #[sqlx(default)]
    pub app_id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 更新时间
    #[sqlx(default)]
    pub change_time: u64,

    /// 创建时间
    #[sqlx(default)]
    pub create_time: u64,

    #[sqlx(default)]
    pub status:i8,

    #[sqlx(default)]
    pub barcode_type: String,

    #[sqlx(default)]
    pub image_format: String,

    #[sqlx(default)]
    pub image_width: i32,

    #[sqlx(default)]
    pub image_height: i32,

    #[sqlx(default)]
    pub margin: i32,

    #[sqlx(default)]
    pub image_color: String,

    #[sqlx(default)]
    pub image_background: String,

}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "barcode_parse")]
pub struct BarcodeParseModel {
    /// id
    #[sqlx(default)]
    pub id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// app_id
    #[sqlx(default)]
    pub app_id: u64,

    #[sqlx(default)]
    pub file_hash: String,

    #[sqlx(default)]
    pub barcode_type: String,

    #[sqlx(default)]
    pub record: String,

    /// 解析时间
    #[sqlx(default)]
    pub create_time: u64,

    #[sqlx(default)]
    pub status:i8,

    /// 删除时间
    #[sqlx(default)]
    pub change_time: u64,

}
