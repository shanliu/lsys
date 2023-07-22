use serde::Deserialize;
use serde_json::json;

use crate::{dao::WebDao, JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct AreaCodeParam {
    pub code: String,
}

pub async fn area_list(param: AreaCodeParam, web_dao: &WebDao) -> JsonResult<JsonData> {
    let data = web_dao
        .area
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

pub async fn area_detail(param: AreaCodeParam, web_dao: &WebDao) -> JsonResult<JsonData> {
    let data = web_dao
        .area
        .code_detail(&param.code)?
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

#[derive(Debug, Deserialize)]
pub struct AreaSearchParam {
    pub key_word: String,
}

pub async fn area_search(param: AreaSearchParam, web_dao: &WebDao) -> JsonResult<JsonData> {
    let data = web_dao
        .area
        .code_search(&param.key_word, 10)?
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
