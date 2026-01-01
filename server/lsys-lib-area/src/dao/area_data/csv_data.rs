use std::path::PathBuf;

use csv::{ReaderBuilder, StringRecord};
use unicode_segmentation::UnicodeSegmentation;

use crate::{
    dao::utils::name_clear, AreaCodeData, AreaDataProvider, AreaError, AreaGeoData,
    AreaGeoDataItem, AreaResult,
};

use super::utils::{de_gz_data, en_name_keyword, read_file, read_file_md5};
impl From<std::io::Error> for AreaError {
    fn from(err: std::io::Error) -> Self {
        AreaError::DB(format!("data file error:{:?}", err))
    }
}
pub struct CsvAreaCodeData {
    csv_data: PathBuf,
    gz: bool,
    skip: usize,        //跳过头部n行
    column_name: u8,    //城市完整名字段，从0开始
    column_code: u8,    //城市编码字段，从0开始
    column_hide: u8,    //是否隐藏字段，可忽略
    column_keyword: i8, //关键字字段
    column_enname: i8,  //英文名字段
}

impl CsvAreaCodeData {
    pub fn from_path(
        csv_data: PathBuf,
        skip: usize,
        column_name: u8,
        column_code: u8,
        column_hide: u8,
        column_keyword: i8,
        column_enname: i8,
    ) -> Self {
        Self {
            csv_data,
            gz: false,
            skip,
            column_name,
            column_code,
            column_hide,
            column_keyword,
            column_enname,
        }
    }
    fn create_inner_data(csv_data: PathBuf, gz: bool) -> Self {
        Self {
            csv_data,
            gz,
            skip: 1,
            column_code: 0,
            column_name: 1,
            column_hide: 4,
            column_keyword: 2,
            column_enname: 3,
        }
    }
    /// csv_data csv文件路径
    /// gz 是否压缩文件
    pub fn from_inner_path(csv_data: PathBuf, gz: bool) -> AreaResult<Self> {
        std::fs::metadata(&csv_data)?;
        Ok(Self::create_inner_data(csv_data, gz))
    }
}

pub struct CsvAreaGeoData {
    csv_data: PathBuf,
    gz: bool,
    skip: usize,          //跳过头部几行
    column_code: u8,      //城市编码字段，从0开始
    column_center: u8,    //城市中心坐标字段，不存在时从范围中取中心，从0开始
    column_polygon: u8,   //坐标范围字段，从0开始
    code_len: Vec<usize>, //CODE 长度限制
}

impl CsvAreaGeoData {
    pub fn from_path(
        csv_data: PathBuf,
        skip: usize,
        column_code: u8,
        column_center: u8,
        column_polygon: u8,
        code_len: Vec<usize>,
    ) -> Self {
        Self {
            gz: false,
            csv_data,
            skip,
            column_code,
            column_center,
            column_polygon,
            code_len,
        }
    }
    fn create_inner_data(csv_data: PathBuf, gz: bool) -> Self {
        Self {
            csv_data,
            gz,
            skip: 1,
            column_code: 0,
            column_center: 1,
            column_polygon: 2,
            code_len: vec![1, 6],
        }
    }
    /// csv_data csv文件路径
    /// gz 是否压缩文件
    pub fn from_inner_path(csv_data: PathBuf, gz: bool) -> AreaResult<Self> {
        std::fs::metadata(&csv_data)?;
        Ok(Self::create_inner_data(csv_data, gz))
    }
}

pub struct CsvAreaData {
    code_config: CsvAreaCodeData,
    geo_config: Option<CsvAreaGeoData>,
}
impl CsvAreaData {
    pub fn new(code_config: CsvAreaCodeData, geo_config: Option<CsvAreaGeoData>) -> Self {
        Self {
            code_config,
            geo_config,
        }
    }
    fn read_data<T>(
        &self,
        csv_data: &[u8],
        skip: usize,
        f: impl Fn(&StringRecord) -> Option<T>,
    ) -> AreaResult<Vec<T>> {
        let mut rdr = ReaderBuilder::new()
            .has_headers(false)
            .from_reader(csv_data);
        let mut out = vec![];
        for (i, result) in rdr.records().enumerate() {
            if i < skip {
                continue;
            }
            let record = result.map_err(|e| AreaError::DB(e.to_string()))?;
            if let Some(item) = f(&record) {
                out.push(item)
            }
        }
        Ok(out)
    }
}
impl AreaDataProvider for CsvAreaData {
    fn code_data(&self) -> AreaResult<Vec<AreaCodeData>> {
        let mut csv_data = read_file(&self.code_config.csv_data)?;
        if self.code_config.gz {
            csv_data = de_gz_data(csv_data)?;
        }

        self.read_data(&csv_data, self.code_config.skip, |row| {
            if let Some(code) = row.get(self.code_config.column_code as usize) {
                if code.is_empty() {
                    return None;
                }
                let hide = row.get(self.code_config.column_hide as usize).unwrap_or("");
                let name = row
                    .get(self.code_config.column_name as usize)
                    .unwrap_or_default();
                let mut key_word = "".to_string();
                if self.code_config.column_keyword >= 0 {
                    let word = row
                        .get(self.code_config.column_keyword as usize)
                        .unwrap_or("")
                        .trim();
                    if !word.is_empty() {
                        key_word = word.unicode_words().collect::<Vec<&str>>().join(" ");
                    }
                }
                let mut en_key_word = "".to_string();
                if self.code_config.column_enname >= 0 {
                    let en_name = row
                        .get(self.code_config.column_enname as usize)
                        .unwrap_or("")
                        .trim();
                    if !en_name.is_empty() {
                        en_key_word =
                            en_name.to_lowercase() + " " + en_name_keyword(en_name).as_str();
                    }
                }
                let item = AreaCodeData {
                    code: code.to_owned(),
                    hide: hide == "1" || name.is_empty(),
                    name: name_clear(name),
                    key_word: key_word + " " + en_key_word.as_str(),
                };
                Some(item)
            } else {
                None
            }
        })
    }
    fn geo_data(&self) -> AreaResult<Vec<AreaGeoData>> {
        match &self.geo_config {
            Some(geo_config) => {
                let mut csv_data = read_file(&geo_config.csv_data)?;
                if geo_config.gz {
                    csv_data = de_gz_data(csv_data)?;
                }

                let out = self.read_data(&csv_data, geo_config.skip, |row| {
                    if let Some(code) = row.get(geo_config.column_code as usize) {
                        if !geo_config.code_len.contains(&code.len()) {
                            return None;
                        }
                        let center = row
                            .get(geo_config.column_center as usize)
                            .unwrap_or("")
                            .to_owned();
                        let polygon = row
                            .get(geo_config.column_polygon as usize)
                            .unwrap_or("")
                            .to_owned();
                        return Some(AreaGeoData {
                            code: code.to_owned(),
                            item: vec![AreaGeoDataItem { center, polygon }],
                        });
                    }
                    None
                })?;
                Ok(out)
            }
            None => Ok(vec![]),
        }
    }
    fn code_data_version(&self) -> String {
        read_file_md5(&self.code_config.csv_data)
    }
    fn geo_data_version(&self) -> String {
        if let Some(geo_config) = &self.geo_config {
            return read_file_md5(&geo_config.csv_data);
        }
        "".to_string()
    }
}
