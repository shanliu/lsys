use serde::{Deserialize, Serialize};
use sqlx_model::sqlx_model_status;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "u8")]
pub enum BarcodeOutputCodeFormat {
    QrCode = 1,
    BrCode = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "u8")]
pub enum BarcodeOutputImageFormat {
    Png = 1,
    Jpg = 2,
    JsonPng = 3,
    JsonJpg = 4,
}
