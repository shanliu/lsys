mod area_data;
mod utils;
pub use area_data::*;
mod area_code;
mod area_store;
pub use area_code::*;
pub use area_store::*;
mod area_geo;
pub use area_geo::*;
use tantivy::{directory::error::OpenDirectoryError, query::QueryParserError, TantivyError};
mod area;
pub use area::*;
use std::fmt::{Display, Formatter};
pub enum AreaDao {
    #[cfg(feature = "data-csv")]
    CsvMem(Area<AreaStoreMemory, CsvAreaData>),
    #[cfg(feature = "data-sqlite")]
    SqliteMem(Area<AreaStoreMemory, SqliteAreaData>),
    #[cfg(feature = "data-mysql")]
    MysqlMem(Area<AreaStoreMemory, MysqlAreaData>),
    #[cfg(all(feature = "data-csv", feature = "index-disk"))]
    CsvDisk(Area<AreaStoreDisk, CsvAreaData>),
    #[cfg(all(feature = "data-sqlite", feature = "index-disk"))]
    SqliteDisk(Area<AreaStoreDisk, SqliteAreaData>),
    #[cfg(all(feature = "data-mysql", feature = "index-disk"))]
    MysqlDisk(Area<AreaStoreDisk, MysqlAreaData>),
}
macro_rules! proxy_method {
    ($method:ident,[$($arg:ident:$arg_type:ty),*],$ret:ty) => {
        pub fn $method(&self, $($arg:$arg_type),*) -> $ret {
            match self {
                #[cfg(feature = "data-csv")]
                AreaDao::CsvMem(tmp) => tmp.$method($($arg),*),
                #[cfg(feature = "data-sqlite")]
                AreaDao::SqliteMem(tmp) => tmp.$method($($arg),*),
                #[cfg(feature = "data-mysql")]
                AreaDao::MysqlMem(tmp) => tmp.$method($($arg),*),
                #[cfg(all(feature = "data-csv", feature = "index-disk"))]
                AreaDao::CsvDisk(tmp) => tmp.$method($($arg),*),
                #[cfg(all(feature = "data-sqlite", feature = "index-disk"))]
                AreaDao::SqliteDisk(tmp) => tmp.$method($($arg),*),
                #[cfg(all(feature = "data-mysql", feature = "index-disk"))]
                AreaDao::MysqlDisk(tmp) => tmp.$method($($arg),*),
            }
        }
    };
}

impl AreaDao {
    #[cfg(feature = "data-mysql")]
    pub fn from_mysql_mem(data: MysqlAreaData, store: AreaStoreMemory) -> AreaResult<Self> {
        Ok(Self::MysqlMem(Area::new(store, data)?))
    }
    #[cfg(feature = "data-sqlite")]
    pub fn from_sqlite_mem(data: SqliteAreaData, store: AreaStoreMemory) -> AreaResult<Self> {
        Ok(Self::SqliteMem(Area::new(store, data)?))
    }
    #[cfg(feature = "data-csv")]
    pub fn from_csv_mem(data: CsvAreaData, store: AreaStoreMemory) -> AreaResult<Self> {
        Ok(Self::CsvMem(Area::new(store, data)?))
    }
    #[cfg(all(feature = "data-mysql", feature = "index-disk"))]
    pub fn from_mysql_disk(data: MysqlAreaData, store: AreaStoreDisk) -> AreaResult<Self> {
        Ok(Self::MysqlDisk(Area::new(store, data)?))
    }
    #[cfg(all(feature = "data-sqlite", feature = "index-disk"))]
    pub fn from_sqlite_disk(data: SqliteAreaData, store: AreaStoreDisk) -> AreaResult<Self> {
        Ok(Self::SqliteDisk(Area::new(store, data)?))
    }
    #[cfg(all(feature = "data-csv", feature = "index-disk"))]
    pub fn from_csv_disk(data: CsvAreaData, store: AreaStoreDisk) -> AreaResult<Self> {
        Ok(Self::CsvDisk(Area::new(store, data)?))
    }
    proxy_method!(code_find, [code:&str],AreaResult<Vec<AreaCodeItem>>);
    proxy_method!(code_related, [code:&str],AreaResult<Vec<Vec<AreaCodeRelatedItem>>>);
    proxy_method!(code_childs, [code:&str],AreaResult<Vec<AreaCodeItem>>);
    proxy_method!(code_search, [name: &str, limit: usize],AreaResult<Vec<AreaSearchItem>>);
    proxy_method!(geo_search, [ lat: f64, lng: f64],AreaResult<Vec<AreaCodeItem>>);
    proxy_method!(geo_reload, [], AreaResult<()>);
    proxy_method!(code_reload, [], AreaResult<()>);
}

//公共结构定义
#[derive(Debug)]
pub enum AreaError {
    DB(String),
    System(String),
    NotFind(String),
    Tantivy(String),
    Store(String),
}
impl Display for AreaError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<TantivyError> for AreaError {
    fn from(err: TantivyError) -> Self {
        AreaError::Tantivy(err.to_string())
    }
}
impl From<QueryParserError> for AreaError {
    fn from(err: QueryParserError) -> Self {
        AreaError::Tantivy(err.to_string())
    }
}
impl From<OpenDirectoryError> for AreaError {
    fn from(err: OpenDirectoryError) -> Self {
        AreaError::Tantivy(err.to_string())
    }
}

pub type AreaResult<T> = Result<T, AreaError>;

#[cfg(feature = "data-csv")]
#[test]
fn test_code() {
    let code_path = std::path::PathBuf::from(format!(
        "{}/data/2023-7-area-code.csv.gz",
        env!("CARGO_MANIFEST_DIR")
    ));
    let geo_data = {
        let geo_path = std::path::PathBuf::from(format!(
            "{}/data/2023-7-area-geo.csv.gz",
            env!("CARGO_MANIFEST_DIR")
        ));
        Some(crate::CsvAreaGeoData::from_inner_path(geo_path, true).unwrap())
    };
    let data = CsvAreaData::new(
        crate::CsvAreaCodeData::from_inner_path(code_path, true).unwrap(),
        geo_data,
    );
    let area = crate::AreaDao::from_csv_mem(data, AreaStoreMemory::default()).unwrap();
    let res = area.code_find("4414").unwrap();
    assert_eq!(res[1].code, "4414");
    let res = area.code_childs("").unwrap();
    assert!(res.iter().any(|e| e.code == "44"));
    let res = area.code_related("441403131203").unwrap();
    assert_eq!(res.len(), 5);
    let res = area.code_search("广东 梅州 南口", 10).unwrap();
    assert_eq!(res[0].item[1].code, "4414");
    let res = area.geo_search(22.57729, 113.89409).unwrap();
    assert_eq!(res[2].code, "440306");
}
