use crate::common::JsonError;
use crate::common::{JsonData, JsonResult};
use lsys_core::{
    fluent_message, string_clear, valid_key, StringClear, ValidParam, ValidParamCheck,
    ValidPattern, ValidStrlen,
};
use lsys_core::{AppCore, AppCoreError, IntoFluentMessage};
use lsys_lib_area::{AreaCodeItem, AreaCodeRelatedItem, AreaDao, AreaSearchItem};
use std::sync::Arc;
use tracing::{error, info, warn};
pub struct AppArea {
    area: Option<AreaDao>,
}

impl AppArea {
    pub fn new(app_core: Arc<AppCore>) -> Result<Self, AppCoreError> {
        let area = match app_core.config_path(app_core.config.find(None), "area_code_db") {
            Ok(code_path) => {
                match lsys_lib_area::CsvAreaCodeData::from_inner_path(code_path.clone(), true) {
                    Ok(tmp) => {
                        let geo_data = match app_core
                            .config_path(app_core.config.find(None), "area_geo_db")
                        {
                            Ok(geo_path) => {
                                match lsys_lib_area::CsvAreaGeoData::from_inner_path(
                                    geo_path.clone(),
                                    true,
                                ) {
                                    Ok(geo_obj) => Some(geo_obj),
                                    Err(err) => {
                                        warn!("area code db load fail on {} [download url: https://github.com/shanliu/lsys/releases/tag/v0.0.0 2023-7-area-geo.csv.gz ],error detail:{}",geo_path.display(),err);
                                        None
                                    }
                                }
                            }
                            Err(err) => {
                                info!(
                                    "area geo config load fail :{}",
                                    err.to_fluent_message().default_format()
                                );
                                None
                            }
                        };
                        let data = lsys_lib_area::CsvAreaData::new(tmp, geo_data);
                        let area_index_dir = app_core
                            .config_path(app_core.config.find(None), "area_index_dir")
                            .unwrap_or_else(|_| {
                                let mut index_dir = std::env::temp_dir();
                                index_dir.push("lsys_area_cache");
                                index_dir
                            });
                        let area_index_size = app_core
                            .config
                            .find(None)
                            .get_int("area_index_size")
                            .map(|e| e.unsigned_abs() as usize)
                            .ok();
                        let area_store =
                            lsys_lib_area::AreaStoreDisk::new(area_index_dir, area_index_size)
                                .map_err(|e| AppCoreError::System(e.to_string()))?;
                        Some(
                            AreaDao::from_csv_disk(data, area_store)
                                .map_err(|e| AppCoreError::System(e.to_string()))?,
                        )
                    }
                    Err(err) => {
                        warn!("area code db load fail on {} [download url: https://github.com/shanliu/lsys/releases/tag/v0.0.0 2023-7-area-code.csv.gz ],error detail:{}",code_path.display(),err);
                        None
                    }
                }
            }
            Err(err) => {
                error!(
                    "load area config fail:{}",
                    err.to_fluent_message().default_format()
                );
                None
            }
        };
        Ok(Self { area })
    }
}

macro_rules! get_area {
    ($area:expr) => {
        match $area.as_ref() {
            Some(area) => area,
            None => {
                return Err(JsonError::JsonResponse(
                    JsonData::default(),
                    fluent_message!("area-not-enable"),
                ));
            }
        }
    };
}

impl AppArea {
    fn code_param(&self, code: &str) -> JsonResult<()> {
        ValidParam::default()
            .add(
                valid_key!("area_code"),
                &code,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::Numeric)
                    .add_rule(ValidStrlen::max(12)),
            )
            .check()?;
        Ok(())
    }
    pub fn code_find(&self, code: &str) -> JsonResult<Vec<AreaCodeItem>> {
        self.code_param(code)?;
        Ok(get_area!(self.area).code_find(code)?)
    }
    pub fn code_related(&self, code: &str) -> JsonResult<Vec<Vec<AreaCodeRelatedItem>>> {
        self.code_param(code)?;
        Ok(get_area!(self.area).code_related(code)?)
    }
    pub fn code_childs(&self, code: &str) -> JsonResult<Vec<AreaCodeItem>> {
        self.code_param(code)?;
        Ok(get_area!(self.area).code_childs(code)?)
    }
    pub fn code_search(&self, name: &str, limit: usize) -> JsonResult<Vec<AreaSearchItem>> {
        let name = string_clear(name, StringClear::LikeKeyWord, Some(255));
        Ok(get_area!(self.area).code_search(&name, limit)?)
    }
    pub fn geo_search(&self, lat: f64, lng: f64) -> JsonResult<Vec<AreaCodeItem>> {
        Ok(get_area!(self.area).geo_search(lat, lng)?)
    }
}
