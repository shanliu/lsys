use serde::Deserialize;
use serde_json::json;

use crate::{
    common::RequestDao,
    common::{JsonData, JsonResult},
};

#[derive(Debug, Deserialize)]
pub struct AreaCodeParam {
    pub code: String,
}

pub fn area_list(param: &AreaCodeParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .app_area
        .code_childs(&param.code)?
        .into_iter()
        .map(|e| {
            json!({
                "name":e.name,
                "code":e.code,
                "leaf":e.leaf,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "area": data })))
}

pub fn area_related(param: &AreaCodeParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .app_area
        .code_related(&param.code)?
        .into_iter()
        .map(|e| {
            e.into_iter()
                .map(|e| {
                    json!({
                        "name":e.item.name,
                        "code":e.item.code,
                        "leaf":e.item.leaf,
                        "selected":e.selected,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "area": data })))
}

pub fn area_find(param: &AreaCodeParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .app_area
        .code_find(&param.code)?
        .into_iter()
        .map(|e| {
            json!({
                "name":e.name,
                "code":e.code,
                "leaf":e.leaf,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "area": data })))
}

#[derive(Debug, Deserialize)]
pub struct AreaSearchParam {
    pub key_word: String,
    pub limit: Option<usize>,
}
pub fn area_search(param: &AreaSearchParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .app_area
        .code_search(&param.key_word, param.limit.unwrap_or(10))?
        .into_iter()
        .map(|e| {
            e.item
                .into_iter()
                .map(|e| {
                    json!({
                        "name":e.name,
                        "code":e.code,
                        "leaf":e.leaf,
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "area": data })))
}
#[derive(Debug, Deserialize)]
pub struct AreaGeoParam {
    pub lat: f64,
    pub lng: f64,
}

pub fn area_geo(param: &AreaGeoParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .app_area
        .geo_search(param.lat, param.lng)?
        .into_iter()
        .map(|e| {
            json!({
                "name":e.name,
                "code":e.code,
                "leaf":e.leaf,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({ "area": data })))
}
