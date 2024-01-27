use serde::Deserialize;
use serde_json::json;

use crate::{dao::RequestDao, JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct AreaCodeParam {
    pub code: String,
}

pub fn area_list(param: AreaCodeParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = get_area!(req_dao.web_dao.area)
        .code_childs(&param.code)
        .map_err(|e| req_dao.fluent_json_data(e))?
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

pub fn area_detail(param: AreaCodeParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = get_area!(req_dao.web_dao.area)
        .code_related(&param.code)
        .map_err(|e| req_dao.fluent_json_data(e))?
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

pub fn area_search(param: AreaSearchParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = get_area!(req_dao.web_dao.area)
        .code_search(&param.key_word, 10)
        .map_err(|e| req_dao.fluent_json_data(e))?
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
