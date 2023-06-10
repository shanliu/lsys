use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_files::NamedFile;
use actix_web::error::InternalError;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use actix_web::web::Data;
use actix_web::CustomizeResponder;
use actix_web::Responder;
use actix_web::Result;
use lsys_web::dao::WebDao;
use lsys_web::handler::api::setting::docs_add;
use lsys_web::handler::api::setting::docs_file;
use lsys_web::handler::api::setting::docs_git_detail;
use lsys_web::handler::api::setting::docs_list;
use lsys_web::handler::api::setting::docs_logs;
use lsys_web::handler::api::setting::docs_md_read;
use lsys_web::handler::api::setting::docs_menu;

use lsys_web::handler::api::setting::DocsAddParam;
use lsys_web::handler::api::setting::DocsGitDetailParam;
use lsys_web::handler::api::setting::DocsLogsParam;
use lsys_web::handler::api::setting::DocsMdReadParam;
use lsys_web::handler::api::setting::DocsRawReadParam;
use reqwest::StatusCode;

#[get("/raw/{id}/{path}")]
async fn docs_raw(
    web_dao: Data<WebDao>,
    info: web::Path<(u64, String)>,
) -> Result<CustomizeResponder<NamedFile>> {
    let path = docs_file(
        &DocsRawReadParam {
            menu_id: info.0,
            url: info.1.to_owned(),
        },
        &web_dao,
    )
    .await
    .map_err(|e| InternalError::new(e.to_string(), StatusCode::OK))?;

    let file = NamedFile::open_async(path.path)
        .await?
        .customize()
        .insert_header(("x-version", path.version));

    Ok(file)
}

#[post("/setting/{type}")]
pub async fn docs_setting(
    path: actix_web::web::Path<(String,)>,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = match path.0.to_string().as_str() {
        "edit" => docs_add(json_param.param::<DocsAddParam>()?, &auth_dao).await,
        "list" => docs_list(&auth_dao).await,
        "logs" => docs_logs(json_param.param::<DocsLogsParam>()?, &auth_dao).await,
        "git_detail" => docs_git_detail(json_param.param::<DocsGitDetailParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}

#[post("/read/{type}")]
pub async fn docs_read(
    path: actix_web::web::Path<(String,)>,
    web_dao: Data<WebDao>,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.0.to_string().as_str() {
        "menu" => docs_menu(&web_dao).await,
        "md" => docs_md_read(json_param.param::<DocsMdReadParam>()?, &web_dao).await,
        name => handler_not_found!(name),
    };
    Ok(res?.into())
}
