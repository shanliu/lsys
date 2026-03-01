mod barcode;
mod cache;
mod logger;
mod result;

pub use barcode::ParseParam;
pub use result::*;

use std::{path::Path, sync::Arc};

use image::{ImageBuffer, ImageFormat, Rgb};
use lsys_logger::dao::ChangeLoggerDao;
use rxing::{BarcodeFormat, RXingResult, ResultPoint};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{MySql, Pool};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
};
use tracing::warn;

use crate::model::{
    BarcodeCreateModel, BarcodeCreateStatus, BarcodeParseModel, BarcodeParseStatus,
};
use barcode::BarCodeCore;
use logger::LogBarCodeParseRecord;
use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    db::SqlSuffix,
    fluent_message,
};
use lsys_core::{db::OffsetPageParam, sql_format};
use lsys_core::db::SqlQuote;
use lsys_core::db::{BatchInsert, Insert, SqlExpr, TableMeta, Update};
use lsys_core::remote_notify::RemoteNotify;
use lsys_core::utils::{now_time, string_clear, RequestEnv, StringClear, STRING_CLEAR_FORMAT};
use lsys_core::valid_key;
use lsys_core::valid_param::{
    ValidColor, ValidContains, ValidNumber, ValidParam, ValidParamCheck, ValidStrlen,
};

use crate::dao::logger::LogBarCodeCreateConfig;
use sha2::Digest;
use sha2::Sha256;

pub struct BarCodeConfig {
    pub create_max_len: u64,
    pub create_model_cache: LocalCacheConfig,
    pub create_render_cache: LocalCacheConfig,
}

impl BarCodeConfig {
    pub fn new(create_max_len: u64, use_cache: bool) -> Self {
        Self {
            create_max_len,
            create_model_cache: LocalCacheConfig::new(
                "barcode-create-model",
                if use_cache { None } else { Some(0) },
                None,
            ),
            create_render_cache: LocalCacheConfig::new(
                "barcode-create-render",
                if use_cache { None } else { Some(0) },
                None,
            ),
        }
    }
}

pub struct BarCodeDao {
    db: Pool<MySql>,
    barcode: BarCodeCore,
    logger: Arc<ChangeLoggerDao>,
    create_max_len: u64,
    pub(crate) create_model: Arc<LocalCache<u64, BarcodeCreateModel>>,
    #[allow(clippy::type_complexity)]
    pub(crate) create_render: Arc<LocalCache<String, ImageBuffer<Rgb<u8>, Vec<u8>>>>,
}

impl BarCodeDao {
    pub fn new(
        db: Pool<MySql>,
        remote_notify: Arc<RemoteNotify>,
        config: BarCodeConfig,
        logger: Arc<ChangeLoggerDao>,
    ) -> Self {
        Self {
            db,
            barcode: BarCodeCore::default(),
            create_model: Arc::from(LocalCache::new(
                remote_notify.clone(),
                config.create_model_cache,
            )),
            create_render: Arc::from(LocalCache::new(
                remote_notify.clone(),
                config.create_render_cache,
            )),
            create_max_len: config.create_max_len,
            logger,
        }
    }

    #[allow(dead_code)]
    pub fn log_types() -> Vec<&'static str> {
        use lsys_logger::dao::ChangeLogData;
        vec![
            logger::LogBarCodeCreateConfig::log_type(),
            logger::LogBarCodeParseRecord::log_type(),
        ]
    }
}

impl BarCodeDao {
    pub async fn find_by_create_config_id(&self, id: &u64) -> BarCodeResult<BarcodeCreateModel> {
        Ok(lsys_core::db::utils::fetch_one::<BarcodeCreateModel>(
            &self.db,
            lsys_core::sql_format!("id={id}", id = id),
        ).await?)
    }

    //根据配置,创建一个二维码
    pub async fn create(
        &self,
        barcode_create: &BarcodeCreateModel,
        contents: &str,
    ) -> BarCodeResult<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        if self.create_max_len > 0 {
            ValidParam::default()
                .add(
                    valid_key!("code_contents"),
                    &contents,
                    &ValidParamCheck::default()
                        .add_rule(ValidStrlen::range(1, self.create_max_len)),
                )
                .check()?;
        }
        self.barcode.render(barcode_create, contents)
    }

    pub async fn barcode_show(
        &self,
        contents: &str,
        barcode_create: &BarcodeCreateModel,
        use_cache: bool,
    ) -> BarCodeResult<(ImageFormat, Vec<u8>)> {
        let image_buffer = if use_cache {
            self.cache().create(barcode_create, contents).await?
        } else {
            self.create(barcode_create, contents).await?
        };
        let mut out: Vec<u8> = Vec::new();
        let image_format = ImageFormat::from_extension(&barcode_create.image_format).ok_or(
            BarCodeError::System(
                fluent_message!("barcode-bad-format-error",{"format":&barcode_create.image_format}),
            ),
        )?;
        image_buffer.write_to(&mut std::io::Cursor::new(&mut out), image_format)?;
        Ok((image_format, out))
    }

    async fn find_by_hash(&self, app_id: u64, file_hash: &str) -> sqlx::Result<BarcodeParseModel> {
        let file_hash = string_clear(
            file_hash,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(65),
        );
        sqlx::query_as::<_, BarcodeParseModel>(&sql_format!(
            "select * from {} where app_id={} and file_hash={} AND STATUS IN ({})",
            BarcodeParseModel::table_name(),
            app_id,
            file_hash,
            &[
                BarcodeParseStatus::Succ as i8,
                BarcodeParseStatus::Fail as i8,
            ]
        ))
        .fetch_one(&self.db)
        .await
    }
}

//  --- parse ---
#[derive(Serialize, Deserialize)]
pub struct ParseDataPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Serialize, Deserialize)]
pub struct ParseData {
    pub text: String,
    pub position: Vec<ParseDataPoint>,
}

impl From<RXingResult> for ParseData {
    fn from(value: RXingResult) -> Self {
        let position = value
            .getPoints()
            .iter()
            .map(|e| ParseDataPoint {
                x: e.getX(),
                y: e.getY(),
            })
            .collect::<Vec<_>>();
        Self {
            text: value.getText().to_string(),
            position,
        }
    }
}

pub enum BarcodeParseRecord {
    Succ((BarcodeParseModel, ParseData)),
    Fail(BarcodeParseModel),
}

impl BarCodeDao {
    //解析一个二维码
    pub async fn parse(
        &self,
        user_id: u64,
        app_id: u64,
        file_name: impl AsRef<Path>,
        extension: &str,
        param: &ParseParam<'_>,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<BarcodeParseRecord> {
        let file_hash = compute_file_hash(&file_name).await?;
        match self.find_by_hash(app_id, &file_hash).await {
            Ok(row) => return Ok(parse_model_decode(row)),
            Err(err) => match err {
                sqlx::Error::RowNotFound => {}
                _ => {
                    return Err(err.into());
                }
            },
        };

        match self.barcode.decode(file_name, extension, param).await {
            Ok(data) => {
                let tmps = data
                    .into_iter()
                    .map(|e| {
                        (
                            e.getBarcodeFormat().to_string(),
                            json!(ParseData::from(e)).to_string(),
                        )
                    })
                    .collect::<Vec<(String, String)>>();
                let create_time = now_time().unwrap_or_default();
                let status = BarcodeParseStatus::Succ as i8;
                let mut batch = BatchInsert::<_,BarcodeParseModel>::with_capacity(tmps.len());
                for tmp in tmps.iter() {
                    batch = batch.push(
                        Insert::<_,BarcodeParseModel>::new()
                            .set(BarcodeParseModel::USER_ID, user_id)
                            .set(BarcodeParseModel::APP_ID, app_id)
                            .set(BarcodeParseModel::FILE_HASH, &file_hash)
                            .set(BarcodeParseModel::BARCODE_TYPE, &tmp.0)
                            .set(BarcodeParseModel::RECORD, &tmp.1)
                            .set(BarcodeParseModel::CREATE_TIME, create_time)
                            .set(BarcodeParseModel::STATUS, status),
                    );
                }
                let data_len = tmps.len();
                match batch.execute(&self.db).await {
                    Ok(res) => {
                        self.logger
                            .add(
                                &LogBarCodeParseRecord {
                                    action: "parse",
                                    count: data_len,
                                    message: "succ",
                                    user_id,
                                },
                                Some(res.last_insert_id()),
                                Some(user_id),
                                None,
                                env_data,
                            )
                            .await;
                    }
                    Err(err) => {
                        warn!("add parse record fail:{}", err);
                    }
                };
            }
            Err(err) => {
                let create_time = now_time().unwrap_or_default();
                let barcode_type = "";
                let status = BarcodeParseStatus::Fail as i8;
                match Insert::<_,BarcodeParseModel>::new()
                    .set(BarcodeParseModel::USER_ID, user_id)
                    .set(BarcodeParseModel::APP_ID, app_id)
                    .set(BarcodeParseModel::FILE_HASH, &file_hash)
                    .set(BarcodeParseModel::BARCODE_TYPE, barcode_type)
                    .set(BarcodeParseModel::RECORD, err)
                    .set(BarcodeParseModel::CREATE_TIME, create_time)
                    .set(BarcodeParseModel::STATUS, status)
                    .execute(&self.db)
                    .await
                {
                    Ok(res) => {
                        self.logger
                            .add(
                                &LogBarCodeParseRecord {
                                    action: "parse",
                                    count: 0,
                                    message: "fail",
                                    user_id,
                                },
                                Some(res.last_insert_id()),
                                Some(user_id),
                                None,
                                env_data,
                            )
                            .await;
                    }
                    Err(err) => {
                        warn!("add parse record fail:{}", err);
                    }
                };
            }
        };
        Ok(self
            .find_by_hash(app_id, &file_hash)
            .await
            .map(parse_model_decode)?)
    }

    #[allow(clippy::too_many_arguments)]
    async fn config_param_valid(
        &self,
        app_id: u64,
        status: &BarcodeCreateStatus,
        barcode_type: &str,
        image_format: &str,
        image_width: i32,
        image_height: i32,
        margin: i32,
        image_color: &str,
        image_background: &str,
    ) -> BarCodeResult<()> {
        ValidParam::default()
            .add(
                valid_key!("app_id"),
                &app_id,
                &ValidParamCheck::default().add_rule(ValidNumber::<u64>::id()),
            )
            .add(
                valid_key!("barcode_status"),
                &(*status as i8),
                &ValidParamCheck::default().add_rule(ValidContains(&[
                    BarcodeCreateStatus::EnablePrivate as i8,
                    BarcodeCreateStatus::EnablePublic as i8,
                ])),
            )
            .add(
                valid_key!("image_color"),
                &image_color,
                &ValidParamCheck::default().add_rule(ValidColor::RGB),
            )
            .add(
                valid_key!("image_background"),
                &image_background,
                &ValidParamCheck::default().add_rule(ValidColor::RGB),
            )
            .add(
                valid_key!("margin"),
                &margin,
                &ValidParamCheck::default().add_rule(ValidNumber::range(0, 100)),
            )
            .add(
                valid_key!("image_height"),
                &image_height,
                &ValidParamCheck::default().add_rule(ValidNumber::range(10, 10240)),
            )
            .add(
                valid_key!("image_width"),
                &image_width,
                &ValidParamCheck::default().add_rule(ValidNumber::range(10, 10240)),
            )
            .check()?;
        if BarcodeFormat::from(barcode_type) == BarcodeFormat::UNSUPORTED_FORMAT {
            return Err(BarCodeError::System(fluent_message!("barcode-type",{
                "val":barcode_type
            })));
        }
        if ImageFormat::from_extension(image_format).is_none() {
            return Err(BarCodeError::System(fluent_message!("barcode-image",{
                "val":image_format
            })));
        }
        Ok(())
    }

    //创建二维码配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_create_config(
        &self,
        user_id: u64,
        app_id: u64,
        status: &BarcodeCreateStatus,
        barcode_type: &str,
        image_format: &str,
        image_width: i32,
        image_height: i32,
        margin: i32,
        image_color: &str,
        image_background: &str,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<u64> {
        self.config_param_valid(
            app_id,
            status,
            barcode_type,
            image_format,
            image_width,
            image_height,
            margin,
            image_color,
            image_background,
        )
        .await?;

        let create_time = now_time().unwrap_or_default();
        let status = status.to();
        let res = Insert::<_,BarcodeCreateModel>::new()
            .set(BarcodeCreateModel::APP_ID, app_id)
            .set(BarcodeCreateModel::USER_ID, user_id)
            .set(BarcodeCreateModel::CHANGE_USER_ID, user_id)
            .set(BarcodeCreateModel::CREATE_TIME, create_time)
            .set(BarcodeCreateModel::CHANGE_TIME, create_time)
            .set(BarcodeCreateModel::BARCODE_TYPE, barcode_type)
            .set(BarcodeCreateModel::IMAGE_FORMAT, image_format)
            .set(BarcodeCreateModel::IMAGE_WIDTH, image_width)
            .set(BarcodeCreateModel::IMAGE_HEIGHT, image_height)
            .set(BarcodeCreateModel::MARGIN, margin)
            .set(BarcodeCreateModel::IMAGE_COLOR, image_color)
            .set(BarcodeCreateModel::STATUS, status)
            .set(BarcodeCreateModel::IMAGE_BACKGROUND, image_background)
            .execute(&self.db)
            .await?;

        self.logger
            .add(
                &LogBarCodeCreateConfig {
                    action: "add",
                    barcode_type,
                    image_format,
                    image_width,
                    image_height,
                    margin,
                    image_color,
                    image_background,
                    user_id,
                },
                Some(res.last_insert_id()),
                Some(user_id),
                None,
                env_data,
            )
            .await;

        Ok(res.last_insert_id())
    }

    //创建二维码配置
    #[allow(clippy::too_many_arguments)]
    pub async fn edit_create_config(
        &self,
        create_config: &BarcodeCreateModel,
        change_user_id: u64,
        status: &BarcodeCreateStatus,
        barcode_type: &str,
        image_format: &str,
        image_width: i32,
        image_height: i32,
        margin: i32,
        image_color: &str,
        image_background: &str,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<u64> {
        self.config_param_valid(
            create_config.app_id,
            status,
            barcode_type,
            image_format,
            image_width,
            image_height,
            margin,
            image_color,
            image_background,
        )
        .await?;

        let change_time = now_time().unwrap_or_default();
        let status = status.to();
        let row = Update::<_,BarcodeCreateModel>::new()
            .set(BarcodeCreateModel::CHANGE_USER_ID, change_user_id)
            .set(BarcodeCreateModel::CHANGE_TIME, change_time)
            .set(BarcodeCreateModel::BARCODE_TYPE, barcode_type)
            .set(BarcodeCreateModel::IMAGE_FORMAT, image_format)
            .set(BarcodeCreateModel::IMAGE_WIDTH, image_width)
            .set(BarcodeCreateModel::IMAGE_HEIGHT, image_height)
            .set(BarcodeCreateModel::MARGIN, margin)
            .set(BarcodeCreateModel::IMAGE_COLOR, image_color)
            .set(BarcodeCreateModel::STATUS, status)
            .set(BarcodeCreateModel::IMAGE_BACKGROUND, image_background)
            .execute(
                SqlSuffix::Where(&sql_format!("id={}", create_config.id)),
                &self.db,
            )
            .await
            .map(|e| e.rows_affected())?;

        self.logger
            .add(
                &LogBarCodeCreateConfig {
                    action: "edit",
                    barcode_type,
                    image_format,
                    image_width,
                    image_height,
                    margin,
                    user_id: change_user_id,
                    image_color,
                    image_background,
                },
                Some(create_config.id),
                Some(change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(row)
    }

    //删除指定创建二维码配置
    pub async fn delete_create_config(
        &self,
        user_id: u64,
        create_config: &BarcodeCreateModel,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<()> {
        let time = now_time()?;
        Update::<_,BarcodeCreateModel>::new()
            .set(
                BarcodeCreateModel::STATUS,
                BarcodeCreateStatus::Delete as i8,
            )
            .set(BarcodeCreateModel::CHANGE_TIME, time)
            .execute(
                SqlSuffix::Where(&sql_format!("id={}", create_config.id)),
                &self.db,
            )
            .await?;

        self.logger
            .add(
                &LogBarCodeCreateConfig {
                    action: "delete",
                    barcode_type: &create_config.barcode_type,
                    image_format: &create_config.image_format,
                    image_width: create_config.image_width,
                    image_height: create_config.image_height,
                    margin: create_config.margin,
                    image_color: &create_config.image_color,
                    image_background: &create_config.image_background,
                    user_id,
                },
                Some(create_config.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }

    fn list_create_config_where_sql(
        &self,
        user_id: u64,
        id: Option<u64>,
        app_id: Option<u64>,
        barcode_type: Option<&str>,
    ) -> Option<String> {
        let mut sqlwhere = vec![sql_format!(
            "user_id={} and status  in ({})",
            user_id,
            &[
                BarcodeCreateStatus::EnablePrivate as i8,
                BarcodeCreateStatus::EnablePublic as i8
            ]
        )];
        if let Some(s) = app_id {
            sqlwhere.push(sql_format!("app_id={} ", s));
        }
        if let Some(s) = id {
            sqlwhere.push(sql_format!("id={} ", s));
        }
        if let Some(tmp) = barcode_type {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(13));
            if tmp.is_empty() {
                return None;
            }
            sqlwhere.push(sql_format!("barcode_type={} ", tmp));
        }
        Some(sqlwhere.join(" and "))
    }

    //列出创建二维码配置
    pub async fn list_create_config(
        &self,
        user_id: u64,
        id: Option<u64>,
        app_id: Option<u64>,
        barcode_type: Option<&str>,
        page: &OffsetPageParam,
    ) -> BarCodeResult<Vec<BarcodeCreateModel>> {
        let sqlwhere = match self.list_create_config_where_sql(user_id, id, app_id, barcode_type) {
            Some(tmp) => tmp,
            None => return Ok(vec![]),
        };
        Ok(sqlx::query_as::<_, BarcodeCreateModel>(&format!(
            "select * from {} where {} order by id desc {}",
            BarcodeCreateModel::table_name(),
            sqlwhere,
            page.page_query().limit_sql().unwrap_or_default()
        ))
        .fetch_all(&self.db)
        .await?)
    }

    //汇总创建二维码配置
    pub async fn count_create_config(
        &self,
        user_id: u64,
        id: Option<u64>,
        app_id: Option<u64>,
        barcode_type: Option<&str>,
    ) -> BarCodeResult<i64> {
        let sqlwhere = match self.list_create_config_where_sql(user_id, id, app_id, barcode_type) {
            Some(tmp) => tmp,
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select count(*) as total from {} where {}",
            BarcodeCreateModel::table_name(),
            SqlExpr(sqlwhere)
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }

    pub async fn find_by_parse_record_id(&self, id: &u64) -> BarCodeResult<BarcodeParseModel> {
        Ok(lsys_core::db::utils::fetch_one::<BarcodeParseModel>(
            &self.db,
            lsys_core::sql_format!("id={id}", id = id),
        ).await?)
    }

    fn list_parse_record_where_sql(
        &self,
        user_id: u64,
        app_id: Option<u64>,
        barcode_type: Option<&str>,
    ) -> Option<String> {
        let mut sqlwhere = vec![sql_format!(
            "user_id={} and status in ({})",
            user_id,
            &[
                BarcodeParseStatus::Succ as i8,
                BarcodeParseStatus::Fail as i8
            ]
        )];
        if let Some(s) = app_id {
            sqlwhere.push(sql_format!("app_id={} ", s));
        }
        if let Some(tmp) = barcode_type {
            let tmp = string_clear(tmp, StringClear::Option(STRING_CLEAR_FORMAT), Some(13));
            if tmp.is_empty() {
                return None;
            }
            sqlwhere.push(sql_format!("barcode_type={} ", tmp));
        }
        Some(sqlwhere.join(" and "))
    }

    //历史解析的二维码记录
    pub async fn list_parse_record(
        &self,
        user_id: u64,
        app_id: Option<u64>,
        barcode_type: Option<&str>,
        page: &OffsetPageParam,
    ) -> BarCodeResult<Vec<BarcodeParseRecord>> {
        let sqlwhere = match self.list_parse_record_where_sql(user_id, app_id, barcode_type) {
            Some(tmp) => tmp,
            None => return Ok(vec![]),
        };
        Ok(sqlx::query_as::<_, BarcodeParseModel>(&format!(
            "select * from {} where {} order by id desc {}",
            BarcodeParseModel::table_name(),
            sqlwhere,
            page.page_query().limit_sql().unwrap_or_default()
        ))
        .fetch_all(&self.db)
        .await?
        .into_iter()
        .map(parse_model_decode)
        .collect::<_>())
    }

    //汇总历史解析的二维码记录
    pub async fn count_parse_record(
        &self,
        user_id: u64,
        app_id: Option<u64>,
        barcode_type: Option<&str>,
    ) -> BarCodeResult<i64> {
        let sqlwhere = match self.list_parse_record_where_sql(user_id, app_id, barcode_type) {
            Some(tmp) => tmp,
            None => return Ok(0),
        };
        let sql = sql_format!(
            "select count(*) as total from {} where {}",
            BarcodeParseModel::table_name(),
            SqlExpr(sqlwhere)
        );
        let query = sqlx::query_scalar::<_, i64>(&sql);
        let res = query.fetch_one(&self.db).await?;
        Ok(res)
    }

    //删除指定历史解析的二维码记录
    pub async fn delete_parse_record(
        &self,
        user_id: u64,
        parse_record: &BarcodeParseModel,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<()> {
        let time = now_time()?;
        Update::<_,BarcodeParseModel>::new()
            .set(BarcodeParseModel::STATUS, BarcodeParseStatus::Delete as i8)
            .set(BarcodeParseModel::CHANGE_TIME, time)
            .execute(
                SqlSuffix::Where(&sql_format!("id={}", parse_record.id)),
                &self.db,
            )
            .await?;
        self.logger
            .add(
                &LogBarCodeParseRecord {
                    action: "delete",
                    count: 1,
                    message: "succ",
                    user_id,
                },
                Some(parse_record.id),
                Some(user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }

    pub fn cache(&'_ self) -> BarCodeCache<'_> {
        BarCodeCache { dao: self }
    }
}

pub struct BarCodeCache<'t> {
    pub dao: &'t BarCodeDao,
}

impl BarCodeCache<'_> {
    lsys_core::impl_cache_fetch_one!(
        find_by_create_config_id,
        dao,
        create_model,
        u64,
        BarCodeResult<BarcodeCreateModel>
    );

    pub async fn create(
        &self,
        barcode_create: &BarcodeCreateModel,
        contents: &str,
    ) -> BarCodeResult<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        let cont_data = contents.to_owned();
        let key = if cont_data.len() <= 32 {
            format!("#{}", cont_data)
        } else {
            format!("${:x}", md5::compute(&cont_data))
        };
        match self.dao.create_render.get(&key).await {
            Some(data) => Ok(data),
            None => {
                let data = self.dao.create(barcode_create, contents).await?;
                self.dao.create_render.set(key, data.clone(), 0).await;
                Ok(data)
            }
        }
    }
}

fn parse_model_decode(mut s: BarcodeParseModel) -> BarcodeParseRecord {
    if BarcodeParseStatus::Succ.eq(s.status) {
        match serde_json::from_str::<ParseData>(&s.record) {
            Ok(data) => return BarcodeParseRecord::Succ((s, data)),
            Err(err) => {
                s.status = BarcodeParseStatus::Fail as i8;
                s.record = err.to_string();
            }
        };
    }
    BarcodeParseRecord::Fail(s)
}

async fn compute_file_hash(path: impl AsRef<Path>) -> io::Result<String> {
    let mut file = File::open(path).await?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let n = file.read(&mut buffer).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
