use crate::common::JsonData;
use crate::{
    common::RequestDao,
    common::{JsonResponse, JsonResult},
};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct CodeParam {
    pub code: String,
}
#[allow(clippy::result_large_err)]
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
#[allow(clippy::result_large_err)]
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
#[allow(clippy::result_large_err)]
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
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub limit: Option<u64>,
}
#[allow(clippy::result_large_err)]
pub fn search(param: &SearchParam, req_dao: &RequestDao) -> JsonResult<JsonResponse> {
    let limit = param
        .limit
        .map(|e| if e > 100 { 100 } else { e })
        .unwrap_or(10) as usize;
    let data = req_dao
        .web_dao
        .app_area
        .code_search(&param.key_word, limit)?
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
    #[serde(deserialize_with = "crate::common::deserialize_f64")]
    pub lat: f64,
    #[serde(deserialize_with = "crate::common::deserialize_f64")]
    pub lng: f64,
}
#[allow(clippy::result_large_err)]
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
