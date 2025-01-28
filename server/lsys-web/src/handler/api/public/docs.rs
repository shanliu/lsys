use crate::common::{JsonData, JsonResult, RequestDao};
use lsys_docs::dao::DocPath;
use lsys_docs::dao::GitDocResult;
use serde::Deserialize;
use serde_json::json;
use serde_json::Value;
pub async fn docs_menu(req_dao: &RequestDao) -> JsonResult<JsonData> {
    let data = req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .menu()
        .await?
        .into_iter()
        .map(|e| {
            let mut err = None;
            let mut data = None;
            match e.menu_data {
                Ok(tmp) => match serde_json::from_slice::<Value>(&tmp) {
                    Ok(d) => data = Some(d),
                    Err(e) => err = Some(e.to_string()),
                },
                Err(e) => err = Some(e),
            }
            json!({
                "id":e.menu_id,
                "tag_id":e.tag_id,
                "version":e.version,
                "menu_path":e.menu_path,
                "menu_data":data,
                "menu_error":err
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonData::data(json!({
        "data":data,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsMdReadParam {
    pub url: String,
    pub menu_id: u32,
}
pub async fn docs_md_read(param: &DocsMdReadParam, req_dao: &RequestDao) -> JsonResult<JsonData> {
    let (data, dat) = req_dao
        .web_dao
        .web_doc
        .docs_md_read(param.menu_id, &param.url)
        .await?;
    Ok(JsonData::data(json!({
        "id":data.clone_id,
        "version": data.version,
        "data":dat,
    })))
}

#[derive(Debug, Deserialize)]
pub struct DocsRawReadParam {
    pub menu_id: u32,
    pub url: String,
}

pub async fn docs_file(param: &DocsRawReadParam, req_dao: &RequestDao) -> GitDocResult<DocPath> {
    req_dao
        .web_dao
        .web_doc
        .docs_dao
        .docs
        .menu_file(param.menu_id, &param.url)
        .await
}
