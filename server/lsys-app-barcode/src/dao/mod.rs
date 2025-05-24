mod barcode;
mod cache;
mod logger;
mod result;
pub use barcode::ParseParam;
pub use cache::BarCodeLocalCacheClear;
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
    BarcodeCreateModel, BarcodeCreateModelRef, BarcodeCreateStatus, BarcodeParseModel,
    BarcodeParseModelRef, BarcodeParseStatus,
};
use barcode::BarCodeCore;
use logger::LogBarCodeParseRecord;
use lsys_core::{
    cache::{LocalCache, LocalCacheConfig},
    fluent_message, now_time, PageParam, RemoteNotify, RequestEnv, ValidColor, ValidContains,
    ValidNumber,
};
use lsys_core::{db::SqlQuote, STRING_CLEAR_FORMAT};
use lsys_core::{
    db::{Insert, ModelTableName, SqlExpr, Update},
    string_clear, StringClear,
};
use lsys_core::{model_option_set, sql_format};
use lsys_core::{valid_key, ValidParam, ValidParamCheck, ValidStrlen};

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
}
impl BarCodeDao {
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_create_config_id,
        u64,
        BarcodeCreateModel,
        BarCodeResult<BarcodeCreateModel>,
        id,
        "id={id}"
    );
    //根据配置,创建一个二维码
    pub async fn create(
        &self,
        barcode_create: &BarcodeCreateModel,
        contents: &str,
    ) -> BarCodeResult<ImageBuffer<Rgb<u8>, Vec<u8>>> {
        ValidParam::default()
            .add(
                valid_key!("code_contents"),
                &contents,
                &ValidParamCheck::default().add_rule(ValidStrlen::range(1, self.create_max_len)),
            )
            .check()?;
        self.barcode.render(barcode_create, contents)
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
                let mut datas = Vec::with_capacity(tmps.len());
                for tmp in tmps.iter() {
                    datas.push(model_option_set!(BarcodeParseModelRef, {
                        user_id:user_id,
                        app_id:app_id,
                        file_hash:file_hash,
                        barcode_type:tmp.0,
                        record:tmp.1,
                        create_time:create_time,
                        status:status,
                    }));
                }
                let data_len = datas.len();
                match Insert::<BarcodeParseModel, _>::new_vec(datas)
                    .execute(&self.db)
                    .await
                {
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
                let barcode_type = "".to_owned();
                let status = BarcodeParseStatus::Fail as i8;
                let data = model_option_set!(BarcodeParseModelRef, {
                    user_id:user_id,
                    app_id:app_id,
                    file_hash:file_hash,
                    barcode_type:barcode_type,
                    record:err,
                    create_time:create_time,
                    status:status,
                });
                match Insert::<BarcodeParseModel, _>::new(data)
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
        let image_format = image_format.to_owned();
        let barcode_type = barcode_type.to_owned();
        let image_width = image_width.to_owned();
        let margin = margin.to_owned();
        let image_height = image_height.to_owned();
        let image_color = image_color.to_owned();
        let image_background = image_background.to_owned();
        let status = status.to();
        let data = model_option_set!(BarcodeCreateModelRef, {
            app_id: app_id,
            user_id:user_id,
            change_user_id:user_id,
            create_time:create_time,
            change_time: create_time,
            barcode_type:barcode_type,
            image_format:image_format,
            image_width:image_width,
            image_height:image_height,
            margin:margin,
            image_color:image_color,
            status:status,
            image_background:image_background,
        });
        let res = Insert::<BarcodeCreateModel, _>::new(data)
            .execute(&self.db)
            .await?;

        self.logger
            .add(
                &LogBarCodeCreateConfig {
                    action: "add",
                    barcode_type: &barcode_type,
                    image_format: &image_format,
                    image_width,
                    image_height,
                    margin,
                    image_color: &image_color,
                    image_background: &image_background,
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
        let image_format = image_format.to_owned();
        let barcode_type = barcode_type.to_owned();
        let image_width = image_width.to_owned();
        let image_height = image_height.to_owned();
        let margin = margin.to_owned();

        let image_color = image_color.to_owned();
        let image_background = image_background.to_owned();
        let status = status.to();
        let change = model_option_set!(BarcodeCreateModelRef, {
            change_user_id:change_user_id,
            change_time:change_time,
            barcode_type:barcode_type,
            image_format:image_format,
            image_width:image_width,
            image_height:image_height,
            margin:margin,
            image_color:image_color,
            status:status,
            image_background:image_background,
        });
        let row = Update::<BarcodeCreateModel, _>::new(change)
            .execute_by_pk(create_config, &self.db)
            .await
            .map(|e| e.rows_affected())?;

        self.logger
            .add(
                &LogBarCodeCreateConfig {
                    action: "edit",
                    barcode_type: &barcode_type,
                    image_format: &image_format,
                    image_width,
                    image_height,
                    margin,
                    user_id: change_user_id,
                    image_color: &image_color,
                    image_background: &image_background,
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
        let change = lsys_core::model_option_set!(BarcodeCreateModelRef,{
            status:BarcodeCreateStatus::Delete as i8,
            change_time: time,
        });
        Update::<BarcodeCreateModel, _>::new(change)
            .execute_by_pk(create_config, &self.db)
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
        page: Option<&PageParam>,
    ) -> BarCodeResult<Vec<BarcodeCreateModel>> {
        let sqlwhere = match self.list_create_config_where_sql(user_id, id, app_id, barcode_type) {
            Some(tmp) => tmp,
            None => return Ok(vec![]),
        };
        let page_sql = if let Some(pdat) = page {
            format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            " order by id desc".to_string()
        };
        Ok(sqlx::query_as::<_, BarcodeCreateModel>(&format!(
            "select * from {} where {} {}",
            BarcodeCreateModel::table_name(),
            sqlwhere,
            page_sql
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
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_parse_record_id,
        u64,
        BarcodeParseModel,
        BarCodeResult<BarcodeParseModel>,
        id,
        "id={id}"
    );
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
        page: Option<&PageParam>,
    ) -> BarCodeResult<Vec<BarcodeParseRecord>> {
        let sqlwhere = match self.list_parse_record_where_sql(user_id, app_id, barcode_type) {
            Some(tmp) => tmp,
            None => return Ok(vec![]),
        };
        let page_sql = if let Some(pdat) = page {
            format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            " order by id desc".to_string()
        };
        Ok(sqlx::query_as::<_, BarcodeParseModel>(&format!(
            "select * from {} where {} {}",
            BarcodeParseModel::table_name(),
            sqlwhere,
            page_sql
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
        let change = lsys_core::model_option_set!(BarcodeParseModelRef,{
            status:BarcodeParseStatus::Delete as i8,
            change_time: time,
        });
        Update::<BarcodeParseModel, _>::new(change)
            .execute_by_pk(parse_record, &self.db)
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
        match self.dao.create_render.get(&cont_data).await {
            Some(data) => Ok(data),
            None => {
                let data = self.dao.create(barcode_create, contents).await?;
                self.dao
                    .create_render
                    .set(contents.to_owned(), data.clone(), 0)
                    .await;
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
