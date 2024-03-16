use sqlx::{MySql, Pool};

use crate::model::{
    BarcodeOutputCodeFormat, BarcodeOutputImageFormat, BarcodeOutputModel, BarcodeOutputModelRef,
};
use lsys_core::now_time;
use sqlx_model::{model_option_set, Insert, Select};
use tokio::task;

mod create;
mod parse;
mod result;
use create::*;
pub use result::*;

pub struct BarCode {
    db: Pool<MySql>,
}

impl BarCode {
    pub fn new(db: Pool<MySql>) -> Self {
        Self { db }
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn output_create(
        &self,
        app_id: &u64,
        code_format: &BarcodeOutputCodeFormat,
        image_format: &BarcodeOutputImageFormat,
        user_id: &u64,
        image_color_front: &u32,
        image_color_background: &u32,
        image_background: String,
    ) -> BarCodeResult<u64> {
        let create_time = now_time().unwrap_or_default();
        let code_format = *code_format as u8;
        let image_format = *image_format as u8;
        let data = model_option_set!(BarcodeOutputModelRef, {
            app_id: *app_id,
            code_format:code_format,
            image_format:image_format,
            image_color_front: image_color_front,
            image_color_background:*image_color_background,
            image_background:image_background,
            user_id:*user_id,
            change_user_id:*user_id,
            change_time: create_time,
        });
        let res = Insert::<sqlx::MySql, BarcodeOutputModel, _>::new(data)
            .execute(&self.db)
            .await?;
        Ok(res.last_insert_id())
    }
    pub async fn output(&self, id: u64, data: &str) -> BarCodeResult<String> {
        let app_res = Select::type_new::<BarcodeOutputModel>()
            .fetch_one_by_scalar_pk::<BarcodeOutputModel, _, _>(id, &self.db)
            .await
            .unwrap();
        task::block_in_place(|| create_bar_code(&app_res, data))
    }
    pub fn parse(_data: &parse::ParseData) -> BarCodeResult<String> {
        Ok("DD".to_string())
    }
}
