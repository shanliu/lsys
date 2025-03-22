mod app_feature;
use crate::common::JsonError;
use crate::common::JsonResult;

use image::ImageFormat;
use lsys_app::dao::AppDao;
use lsys_app_barcode::dao::{BarCodeConfig, BarCodeDao};
use lsys_app_barcode::model::BarcodeCreateModel;
use lsys_app_barcode::model::BarcodeCreateStatus;
use lsys_core::{fluent_message, AppCore, RemoteNotify};
use lsys_logger::dao::ChangeLoggerDao;

use sqlx::{MySql, Pool};
use std::io::Cursor;
use std::sync::Arc;

pub struct AppBarCode {
    pub barcode_dao: BarCodeDao,
    app_dao: Arc<AppDao>,
}

impl AppBarCode {
    pub fn new(
        app_core: Arc<AppCore>,
        app_dao: Arc<AppDao>,
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        let create_max = app_core
            .config
            .find(None)
            .get_int("barcode_create_max")
            .map(|e| if e > 0 { e as u64 } else { 0 })
            .unwrap_or(0);
        let use_cache = app_core
            .config
            .find(None)
            .get_bool("use_cache")
            .unwrap_or(false);

        let barcode_dao = BarCodeDao::new(
            db.clone(),
            remote_notify.clone(),
            BarCodeConfig::new(create_max, use_cache),
            logger.clone(),
        );
        AppBarCode {
            barcode_dao,
            app_dao,
        }
    }
}

impl AppBarCode {
    pub async fn barcode_show(
        &self,
        contents: &str,
        barcode_create: &BarcodeCreateModel,
        use_cache: bool,
    ) -> JsonResult<(ImageFormat, Vec<u8>)> {
        if !BarcodeCreateStatus::EnablePublic.eq(barcode_create.status) {
            return Err(JsonError::Message(fluent_message!(
                "barcode-bad-auth-error"
            )));
        }
        let image_buffer = if use_cache {
            self.barcode_dao
                .cache()
                .create(barcode_create, contents)
                .await?
        } else {
            self.barcode_dao.create(barcode_create, contents).await?
        };
        let mut png_data: Vec<u8> = Vec::new();
        let mut cursor = Cursor::new(&mut png_data);
        let image_format = match ImageFormat::from_extension(&barcode_create.image_format) {
            Some(t) => t,
            None => {
                return Err(JsonError::Message(
                    fluent_message!("barcode-bad-format-error",{
                        "foramt":&barcode_create.image_format
                    }),
                ))
            }
        };
        image_buffer
            .write_to(&mut cursor, image_format)
            .map_err(|e| {
                JsonError::Message(fluent_message!("barcode-bad-image-error",{
                    "err":e
                }))
            })?;
        Ok((image_format, png_data))
    }
}
