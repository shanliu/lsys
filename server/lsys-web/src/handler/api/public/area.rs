use serde::Deserialize;
use serde_json::json;
use crate::common::JsonData;
use crate::{
    common::RequestDao,
    common::{JsonResponse, JsonResult},
};

#[derive(Debug, Deserialize)]
pub struct CodeParam {
    pub code: String,
}

pub fn list_data(param: &CodeParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
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
    Ok(JsonResponse::data(JsonData::body(json!({ "area": data }))))
}

pub fn related_find(param: &CodeParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
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
    Ok(JsonResponse::data(JsonData::body(json!({ "area": data }))))
}

pub fn code_find(param: &CodeParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
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
    Ok(JsonResponse::data(JsonData::body(json!({ "area": data }))))
}

#[derive(Debug, Deserialize)]
pub struct SearchParam {
    pub key_word: String,
    pub limit: Option<usize>,
}
pub fn search(param: &SearchParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
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
    Ok(JsonResponse::data(JsonData::body(json!({ "area": data }))))
}
#[derive(Debug, Deserialize)]
pub struct GeoParam {
    pub lat: f64,
    pub lng: f64,
}

pub fn geo_find(param: &GeoParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
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
    Ok(JsonResponse::data(JsonData::body(json!({ "area": data }))))
}
