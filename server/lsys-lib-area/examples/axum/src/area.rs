use lsys_lib_areab_area::{AreaDao, AreaError, AreaResult};
use serde_json::{json, Value};
use std::{collections::HashMap, sync::Arc};
pub(crate) async fn area_handler(
    path: String,
    params: HashMap<String, String>,
    state: Arc<AreaDao>,
) -> AreaResult<Value> {
    match path.as_str() {
        "list" => {
            let code = params.get("code");
            code_childs(&state, code.unwrap_or(&"".to_owned()))
        }
        "related" => {
            let code = params.get("code");
            code_related(&state, code.unwrap_or(&"".to_owned()))
        }
        "search" => {
            let code = params.get("key_word");
            code_search(&state, code.unwrap_or(&"".to_owned()))
        }
        "find" => {
            let code = params.get("code");
            code_find(&state, code.unwrap_or(&"".to_owned()))
        }
        "geo" => {
            let lat = params
                .get("lat")
                .ok_or(AreaError::System(format!("miss lat {} param", path)))?
                .parse::<f64>()
                .map_err(|e| AreaError::System(format!("parse lat fail: {} ", e)))?;
            let lng = params
                .get("lng")
                .ok_or(AreaError::System(format!("miss lat {} param", path)))?
                .parse::<f64>()
                .map_err(|e| AreaError::System(format!("parse lat fail: {} ", e)))?;
            geo_search(&state, lat, lng)
        }
        _ => Err(AreaError::System(format!(
            "your request path {} not support",
            path
        ))),
    }
}

fn code_childs(area: &AreaDao, code: &str) -> AreaResult<Value> {
    let data = area
        .code_childs(code)?
        .into_iter()
        .map(|e| {
            json!({
                "name":e.name,
                "code":e.code,
                "leaf":e.leaf,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!(data))
}

fn code_related(area: &AreaDao, code: &str) -> AreaResult<Value> {
    let data = area
        .code_related(code)?
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
    Ok(json!(data))
}

fn code_search(area: &AreaDao, key_word: &str) -> AreaResult<Value> {
    let data = area
        .code_search(key_word, 10)?
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
    Ok(json!(data))
}

fn code_find(area: &AreaDao, code: &str) -> AreaResult<Value> {
    let data = area
        .code_find(code)?
        .into_iter()
        .map(|e| {
            json!({
                "name":e.name,
                "code":e.code,
                "leaf":e.leaf,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!(data))
}

fn geo_search(area: &AreaDao, lat: f64, lng: f64) -> AreaResult<Value> {
    let data = area
        .geo_search(lat, lng)?
        .into_iter()
        .map(|e| {
            json!({
                "name":e.name,
                "code":e.code,
                "leaf":e.leaf,
            })
        })
        .collect::<Vec<_>>();
    Ok(json!(data))
}
