use std::path::Path;

use image::{ImageBuffer, ImageFormat, Rgb};
use lsys_logger::dao::ChangeLogger;
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
use lsys_core::{fluent_message, now_time,  PageParam, RequestEnv};
use sqlx_model::{
    model_option_set, sql_format, Insert, ModelTableName, Select, SqlExpr, Update, WhereOption,
};

use super::{
    barcode_core::{BarCodeCore, ParseParam},
    logger::LogBarCodeParseRecord,
    BarCodeError, BarCodeResult,
};
use sqlx_model::SqlQuote;

use crate::dao::logger::LogBarCodeCreateConfig;
use sha2::Digest;
use sha2::Sha256;

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

pub struct BarCodeDao {
    db: Pool<MySql>,
    barcode: BarCodeCore,
    logger: ChangeLogger,
}

pub enum BarcodeParseRecord {
    Succ((BarcodeParseModel,ParseData)),
    Fail(BarcodeParseModel)
}

impl BarCodeDao {
    pub fn new(db: Pool<MySql>) -> Self {
        Self {
            logger: ChangeLogger::new(db.clone()),
            db,
            barcode: BarCodeCore::default(),
        }
    }
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
        barcode_create_id: &u64,
        contents: &str,
    ) -> BarCodeResult<(ImageBuffer<Rgb<u8>, Vec<u8>>, BarcodeCreateModel)> {
        let create = Select::type_new::<BarcodeCreateModel>()
            .fetch_one_by_scalar_pk::<BarcodeCreateModel, _, _>(*barcode_create_id, &self.db)
            .await?;
        self.barcode.render(&create, contents).map(|t| (t, create))
    }
    async fn find_by_hash(&self, app_id: &u64, file_hash: &str) -> sqlx::Result<BarcodeParseModel> {
        let sql = WhereOption::Where(sql_format!("app_id={} and file_hash={} AND STATUS IN ({})", app_id, file_hash,&[
            BarcodeParseStatus::Succ as i8,
            BarcodeParseStatus::Fail as i8,
        ]));
        Select::type_new::<BarcodeParseModel>()
            .fetch_one_by_where::<BarcodeParseModel, _>(&sql, &self.db)
            .await
    }
    //解析一个二维码
    pub async fn parse(
        &self,
        user_id: &u64,
        app_id: &u64,
        file_name: impl AsRef<Path>,
        extension: &str,
        param: &ParseParam,
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
                        user_id:*user_id,
                        app_id:*app_id,
                        file_hash:file_hash,
                        barcode_type:tmp.0,
                        record:tmp.1,
                        create_time:create_time,
                        status:status,
                    }));
                }
                let data_len = datas.len();
                match Insert::<sqlx::MySql, BarcodeParseModel, _>::new_vec(datas)
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
                                },
                                &Some(res.last_insert_id()),
                                &Some(*user_id),
                                &Some(*user_id),
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
                    user_id:*user_id,
                    app_id:*app_id,
                    file_hash:file_hash,
                    barcode_type:barcode_type,
                    record:err,
                    create_time:create_time,
                    status:status,
                });
                match Insert::<sqlx::MySql, BarcodeParseModel, _>::new(data)
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
                                },
                                &Some(res.last_insert_id()),
                                &Some(*user_id),
                                &Some(*user_id),
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
        Ok(self.find_by_hash(app_id, &file_hash).await.map(parse_model_decode)?)
    }
    //创建二维码配置
    #[allow(clippy::too_many_arguments)]
    pub async fn add_create_config(
        &self,
        user_id: &u64,
        app_id: &u64,
        status: &BarcodeCreateStatus,
        barcode_type: &str,
        image_format: &str,
        image_width: &i32,
        image_height: &i32,
        margin: &i32,
        image_color: &str,
        image_background: &str,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<u64> {
        if *app_id == 0 {
            return Err(BarCodeError::System(fluent_message!("barcode-app-id",{
                "val":app_id
            })));
        }
        if BarcodeCreateStatus::Delete == *status {
            return Err(BarCodeError::System(fluent_message!("barcode-bad-status",{
                "val":status.to()
            })));
        }
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
        if *image_width <= 0 || *image_height <= 0 {
            return Err(BarCodeError::System(fluent_message!("barcode-bad-size",{
                "val":format!("{}:{}",image_width,image_height)
            })));
        }
        if !is_hex_color(image_color) {
            return Err(BarCodeError::System(
                fluent_message!("barcode-bad-font-color",{
                    "val":image_color
                }),
            ));
        }
        if !is_hex_color(image_background) {
            return Err(BarCodeError::System(
                fluent_message!("barcode-bad-back-color",{
                    "val":image_background
                }),
            ));
        }

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
            app_id: *app_id,
            user_id:*user_id,
            change_user_id:*user_id,
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
        let res = Insert::<sqlx::MySql, BarcodeCreateModel, _>::new(data)
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
                },
                &Some(res.last_insert_id()),
                &Some(*user_id),
                &Some(*user_id),
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
        change_user_id: &u64,
        status: &BarcodeCreateStatus,
        barcode_type: &str,
        image_format: &str,
        image_width: &i32,
        image_height: &i32,
        margin: &i32,
        image_color: &str,
        image_background: &str,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<u64> {
        if BarcodeCreateStatus::Delete.eq(create_config.status) {
            return Err(BarCodeError::System(fluent_message!("barcode-not-find")));
        }
        if BarcodeCreateStatus::Delete == *status {
            return Err(BarCodeError::System(fluent_message!("barcode-bad-status",{
                "val":status.to()
            })));
        }
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
        if *image_width <= 0 || *image_height <= 0 {
            return Err(BarCodeError::System(fluent_message!("barcode-bad-size",{
                "val":format!("{}:{}",image_width,image_height)
            })));
        }
        if !is_hex_color(image_color) {
            return Err(BarCodeError::System(
                fluent_message!("barcode-bad-font-color",{
                    "val":image_color
                }),
            ));
        }
        if !is_hex_color(image_background) {
            return Err(BarCodeError::System(
                fluent_message!("barcode-bad-back-color",{
                    "val":image_background
                }),
            ));
        }
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
            change_user_id:*change_user_id,
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
        let row = Update::<sqlx::MySql, BarcodeCreateModel, _>::new(change)
            .execute_by_pk(create_config, &self.db)
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
                    image_color,
                    image_background,
                },
                &Some(create_config.id),
                &Some(*change_user_id),
                &Some(*change_user_id),
                None,
                env_data,
            )
            .await;

        Ok(row)
    }
    //删除指定创建二维码配置
    pub async fn delete_create_config(
        &self,
        user_id: &u64,
        create_config: &BarcodeCreateModel,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<()> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(BarcodeCreateModelRef,{
            status:BarcodeCreateStatus::Delete as i8,
            change_time: time,
        });
        Update::<sqlx::MySql, BarcodeCreateModel, _>::new(change)
            .execute_by_pk(create_config, &self.db)
            .await?;

        self.logger
            .add(
                &LogBarCodeCreateConfig {
                    action: "delete",
                    barcode_type: create_config.barcode_type.to_owned(),
                    image_format: create_config.image_format.to_owned(),
                    image_width: create_config.image_width,
                    image_height: create_config.image_height,
                    margin: create_config.margin,
                    image_color: create_config.image_color.to_owned(),
                    image_background: create_config.image_background.to_owned(),
                },
                &Some(create_config.id),
                &Some(*user_id),
                &Some(*user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
    fn list_create_config_where_sql(
        &self,
        user_id: &u64,
        app_id: &Option<u64>,
        barcode_type: &Option<String>,
    ) -> String {
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
        if let Some(s) = barcode_type {
            sqlwhere.push(sql_format!("barcode_type={} ", s));
        }
        sqlwhere.join(" and ")
    }
    //列出创建二维码配置
    pub async fn list_create_config(
        &self,
        user_id: &u64,
        app_id: &Option<u64>,
        barcode_type: &Option<String>,
        page: &Option<PageParam>,
    ) -> BarCodeResult<Vec<BarcodeCreateModel>> {
        let sqlwhere = self.list_create_config_where_sql(user_id, app_id, barcode_type);
        let page_sql = if let Some(pdat) = page {
            format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            " order by id desc".to_string()
        };
        let sql = if !sqlwhere.is_empty() {
            WhereOption::Where(sqlwhere + page_sql.as_str())
        } else {
            WhereOption::None
        };
        Ok(Select::type_new::<BarcodeCreateModel>()
            .fetch_all_by_where::<BarcodeCreateModel, _>(&sql, &self.db)
            .await?)
    }
    //汇总创建二维码配置
    pub async fn count_create_config(
        &self,
        user_id: &u64,
        app_id: &Option<u64>,
        barcode_type: &Option<String>,
    ) -> BarCodeResult<i64> {
        let sqlwhere = self.list_create_config_where_sql(user_id, app_id, barcode_type);
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
        user_id: &u64,
        app_id: &Option<u64>,
        barcode_type: &Option<String>,
    ) -> String {
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
        if let Some(s) = barcode_type {
            sqlwhere.push(sql_format!("barcode_type={} ", s));
        }
        sqlwhere.join(" and ")
    }
    //历史解析的二维码记录
    pub async fn list_parse_record(
        &self,
        user_id: &u64,
        app_id: &Option<u64>,
        barcode_type: &Option<String>,
        page: &Option<PageParam>,
    ) -> BarCodeResult<Vec<BarcodeParseRecord>> {
        let sqlwhere = self.list_parse_record_where_sql(user_id, app_id, barcode_type);
        let page_sql = if let Some(pdat) = page {
            format!(
                " order by id desc limit {} offset {} ",
                pdat.limit, pdat.offset
            )
        } else {
            " order by id desc".to_string()
        };
        let sql = if !sqlwhere.is_empty() {
            WhereOption::Where(sqlwhere + page_sql.as_str())
        } else {
            WhereOption::None
        };
        Ok(Select::type_new::<BarcodeParseModel>()
            .fetch_all_by_where::<BarcodeParseModel, _>(&sql, &self.db)
            .await?.into_iter().map(parse_model_decode).collect::<_>())
    }
    //汇总历史解析的二维码记录
    pub async fn count_parse_record(
        &self,
        user_id: &u64,
        app_id: &Option<u64>,
        barcode_type: &Option<String>,
    ) -> BarCodeResult<i64> {
        let sqlwhere = self.list_parse_record_where_sql(user_id, app_id, barcode_type);
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
        user_id: &u64,
        parse_record: &BarcodeParseModel,
        env_data: Option<&RequestEnv>,
    ) -> BarCodeResult<()> {
        let time = now_time()?;
        let change = sqlx_model::model_option_set!(BarcodeParseModelRef,{
            status:BarcodeParseStatus::Delete as i8,
            change_time: time,
        });
        Update::<sqlx::MySql, BarcodeParseModel, _>::new(change)
            .execute_by_pk(parse_record, &self.db)
            .await?;
        self.logger
            .add(
                &LogBarCodeParseRecord {
                    action: "delete",
                    count: 1,
                    message: "succ",
                },
                &Some(parse_record.id),
                &Some(*user_id),
                &Some(*user_id),
                None,
                env_data,
            )
            .await;
        Ok(())
    }
}


fn parse_model_decode(mut s: BarcodeParseModel) -> BarcodeParseRecord{
    if BarcodeParseStatus::Succ.eq(s.status){
        match serde_json::from_str::<ParseData>(&s.record){
            Ok(data)=>return BarcodeParseRecord::Succ((s,data)),
            Err(err)=>{
                s.status=BarcodeParseStatus::Fail as i8;
                s.record=err.to_string();
            }
        };
    }
    BarcodeParseRecord::Fail(s)
}

fn is_hex_color(s: &str) -> bool {
    if s.len() != 6 {
        return false;
    }
    s.chars().all(|c| c.is_ascii_hexdigit())
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
