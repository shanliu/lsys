use std::path::PathBuf;

use log::warn;

use rusqlite::{Connection, Row};
use unicode_segmentation::UnicodeSegmentation;

use crate::{AreaCodeData, AreaDataProvider, AreaError, AreaGeoData, AreaGeoDataItem, AreaResult};

use super::utils::{en_name_keyword, read_file_md5};

impl From<rusqlite::Error> for AreaError {
    fn from(err: rusqlite::Error) -> Self {
        AreaError::DB(format!("sqlite error:{:?}", err))
    }
}

pub struct SqliteAreaCodeData {
    pub conn: PathBuf,
    pub sql: String,
    pub column_name: String,
    pub column_code: String,
    pub column_hide: String,
    pub column_keyword: String, //关键字字段
    pub column_enname: String,  //英文名字段
}

impl SqliteAreaCodeData {
    pub fn from_path(conn: PathBuf) -> Self {
        Self {
            conn,
            sql: "select name,code,kw_name,kw_py,hide from area_code".to_string(),
            column_name: "name".to_string(),
            column_code: "code".to_string(),
            column_hide: "hide".to_string(),
            column_keyword: "kw_name".to_string(),
            column_enname: "kw_py".to_string(),
        }
    }
}

pub struct SqliteAreaGeoData {
    pub conn: PathBuf,
    pub sql: String,
    pub column_code: String,
    pub column_center: String,
    pub column_polygon: String,
}

impl SqliteAreaGeoData {
    pub fn from_path(conn: PathBuf) -> Self {
        Self {
            conn,
            sql: "select code,center,polygon from area_geo where code like '______%'".to_string(),
            column_code: "code".to_string(),
            column_center: "center".to_string(),
            column_polygon: "polygon".to_string(),
        }
    }
}

pub struct SqliteAreaData {
    code_config: SqliteAreaCodeData,
    geo_config: Option<SqliteAreaGeoData>,
}
impl SqliteAreaData {
    pub fn new(code_config: SqliteAreaCodeData, geo_config: Option<SqliteAreaGeoData>) -> Self {
        Self {
            code_config,
            geo_config,
        }
    }
    fn read_data<T>(
        &self,
        conn: &Connection,
        sql: &str,
        f: impl for<'r> Fn(&'r Row<'r>) -> Option<T>,
    ) -> AreaResult<Vec<T>> {
        let mut stmt = conn.prepare(sql)?;
        let mut out = vec![];
        let mut rows = stmt.query(())?;
        while let Some(row) = rows.next()? {
            if let Some(item) = f(row) {
                out.push(item)
            }
        }
        Ok(out)
    }
}
impl AreaDataProvider for SqliteAreaData {
    fn code_data(&self) -> AreaResult<Vec<AreaCodeData>> {
        let conn = rusqlite::Connection::open(&self.code_config.conn)?;

        self.read_data(&conn, &self.code_config.sql, |row| {
            if let Ok(code) = row.get(self.code_config.column_code.as_str()) {
                let hide = row.get(self.code_config.column_hide.as_str()).unwrap_or(0);
                let name: Result<String, rusqlite::Error> =
                    row.get(self.code_config.column_name.as_str());
                let mut key_word = "".to_string();

                let word = row
                    .get(self.code_config.column_keyword.as_str())
                    .unwrap_or_else(|_| "".to_string());
                if !word.is_empty() {
                    key_word = word.unicode_words().collect::<Vec<&str>>().join(" ");
                }
                let mut en_key_word = "".to_string();
                let en_name = row
                    .get(self.code_config.column_enname.as_str())
                    .unwrap_or_else(|_| "".to_string());
                if !en_name.is_empty() {
                    en_key_word =
                        en_name.to_lowercase() + " " + en_name_keyword(en_name.as_str()).as_str();
                }
                Some(AreaCodeData {
                    code,
                    hide: hide == 1 || name.as_ref().map(|e| e.is_empty()).unwrap_or(true),
                    name: name.unwrap_or_else(|_| "[_._]".to_string()),
                    key_word: key_word + " " + en_key_word.as_str(),
                })
            } else {
                None
            }
        })
    }
    fn geo_data(&self) -> AreaResult<Vec<AreaGeoData>> {
        match &self.geo_config {
            Some(get_config) => {
                let conn = rusqlite::Connection::open(&get_config.conn)?;
                let out = self.read_data(&conn, get_config.sql.as_str(), |row| {
                    match row.get(get_config.column_code.as_str()) {
                        Ok(code) => {
                            if let Ok(polygon) = row.get(get_config.column_polygon.as_str()) {
                                return Some(AreaGeoData {
                                    code,
                                    item: vec![AreaGeoDataItem {
                                        center: row
                                            .get(get_config.column_center.as_str())
                                            .unwrap_or_else(|_| "".to_string()),
                                        polygon,
                                    }],
                                });
                            }
                        }
                        Err(e) => {
                            warn!("geo error:{}", e);
                            return None;
                        }
                    }
                    None
                })?;
                Ok(out)
            }
            None => Ok(vec![]),
        }
    }
    fn code_data_version(&self) -> String {
        read_file_md5(&self.code_config.conn)
    }
    fn geo_data_version(&self) -> String {
        if let Some(geo_config) = &self.geo_config {
            return read_file_md5(&geo_config.conn);
        }
        "".to_string()
    }
}
