use mysql::{prelude::Queryable, Conn, Opts, Row};
use unicode_segmentation::UnicodeSegmentation;

use crate::{AreaCodeData, AreaDataProvider, AreaError, AreaGeoData, AreaGeoDataItem, AreaResult};

use super::utils::en_name_keyword;

impl From<mysql::Error> for AreaError {
    fn from(err: mysql::Error) -> Self {
        AreaError::DB(err.to_string())
    }
}
impl From<mysql::UrlError> for AreaError {
    fn from(err: mysql::UrlError) -> Self {
        AreaError::DB(err.to_string())
    }
}

pub struct MysqlAreaCodeData {
    pub uri: String,
    pub sql: String,
    pub column_name: String,
    pub column_code: String,
    pub column_hide: String,
    pub column_keyword: String, //关键字字段
    pub column_enname: String,  //英文名字段
}

impl MysqlAreaCodeData {
    pub fn from_uri(uri: &str, table_name: Option<&str>) -> Self {
        Self {
            uri: uri.to_owned(),
            sql: format!(
                "select name,code,hide,kw_name,kw_py from {}",
                table_name.unwrap_or("area_code")
            ),
            column_name: "name".to_string(),
            column_code: "code".to_string(),
            column_hide: "hide".to_string(),
            column_keyword: "kw_name".to_string(),
            column_enname: "kw_py".to_string(),
        }
    }
}

pub struct MysqlAreaGeoData {
    pub uri: String,
    pub sql: String,
    pub column_code: String,
    pub column_center: String,
    pub column_polygon: String,
}

impl MysqlAreaGeoData {
    pub fn from_uri(uri: &str, table_name: Option<&str>) -> Self {
        Self {
            uri: uri.to_owned(),
            sql: format!(
                "select code,geo,polygon from {} where id like '______%'",
                table_name.unwrap_or("area_geo")
            ),
            column_code: "code".to_string(),
            column_center: "geo".to_string(),
            column_polygon: "polygon".to_string(),
        }
    }
}

pub struct MysqlAreaData {
    code_config: MysqlAreaCodeData,
    geo_config: Option<MysqlAreaGeoData>,
}
impl MysqlAreaData {
    pub fn new(code_config: MysqlAreaCodeData, geo_config: Option<MysqlAreaGeoData>) -> Self {
        Self {
            code_config,
            geo_config,
        }
    }
}

impl AreaDataProvider for MysqlAreaData {
    fn code_data(&self) -> AreaResult<Vec<AreaCodeData>> {
        let opt = Opts::try_from(self.code_config.uri.as_str())?;
        let mut conn = Conn::new(opt)?;
        let result: Vec<Row> = conn.query(self.code_config.sql.as_str())?;
        Ok(result
            .into_iter()
            .flat_map(|row| {
                if let Some(code) = row.get(self.code_config.column_code.as_str()) {
                    let hide = row.get(self.code_config.column_hide.as_str()).unwrap_or(0);
                    let name: Option<String> = row.get(self.code_config.column_name.as_str());
                    let mut key_word = "".to_string();

                    let word: String = row
                        .get(self.code_config.column_keyword.as_str())
                        .unwrap_or_default();
                    if !word.is_empty() {
                        key_word = word.unicode_words().collect::<Vec<&str>>().join(" ");
                    }
                    let mut en_key_word = "".to_string();
                    let en_name: String = row
                        .get(self.code_config.column_enname.as_str())
                        .unwrap_or_default();
                    if !en_name.is_empty() {
                        en_key_word = en_name.to_lowercase()
                            + " "
                            + en_name_keyword(en_name.as_str()).as_str();
                    }
                    Some(AreaCodeData {
                        code,
                        hide: hide == 1 || name.as_ref().map(|e| e.is_empty()).unwrap_or(true),
                        name: name.unwrap_or_else(|| "[_._]".to_string()),
                        key_word: key_word + " " + en_key_word.as_str(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>())
    }
    fn geo_data(&self) -> AreaResult<Vec<AreaGeoData>> {
        match &self.geo_config {
            Some(get_config) => {
                let opt = Opts::try_from(get_config.uri.as_str())?;
                let mut conn = Conn::new(opt)?;
                let result: Vec<Row> = conn.query(get_config.sql.as_str())?;
                Ok(result
                    .into_iter()
                    .flat_map(|row| {
                        if let Some(code) = row.get(get_config.column_code.as_str()) {
                            if let Some(polygon) = row.get(get_config.column_polygon.as_str()) {
                                return Some(AreaGeoData {
                                    code,
                                    item: vec![AreaGeoDataItem {
                                        center: row
                                            .get(get_config.column_center.as_str())
                                            .unwrap_or_default(),
                                        polygon,
                                    }],
                                });
                            }
                        }
                        None
                    })
                    .collect::<Vec<_>>())
            }
            None => Ok(vec![]),
        }
    }
    fn code_data_version(&self) -> String {
        "".to_string()
    }
    fn geo_data_version(&self) -> String {
        "".to_string()
    }
}
