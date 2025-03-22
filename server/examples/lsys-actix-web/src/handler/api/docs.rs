use crate::common::handler::ReqQuery;
use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};
use actix_files::NamedFile;
use actix_http::StatusCode;
use actix_web::error::InternalError;
use actix_web::get;
use actix_web::post;
use actix_web::web;
use actix_web::web::Data;
use actix_web::CustomizeResponder;
use actix_web::Responder;
use actix_web::Result;
use lsys_web::handler::api::public::docs::{
    file_path, md_read, menu_data, MdReadParam, RawReadParam,
};
use lsys_web::handler::api::system::docs::git_add;
use lsys_web::handler::api::system::docs::git_del;
use lsys_web::handler::api::system::docs::git_detail;
use lsys_web::handler::api::system::docs::git_edit;
use lsys_web::handler::api::system::docs::git_list;
use lsys_web::handler::api::system::docs::menu_add;
use lsys_web::handler::api::system::docs::menu_del;
use lsys_web::handler::api::system::docs::menu_list;
use lsys_web::handler::api::system::docs::tag_add;
use lsys_web::handler::api::system::docs::tag_clone_del;
use lsys_web::handler::api::system::docs::tag_del;
use lsys_web::handler::api::system::docs::tag_dir;
use lsys_web::handler::api::system::docs::tag_file_info;
use lsys_web::handler::api::system::docs::tag_list;
use lsys_web::handler::api::system::docs::tag_logs;
use lsys_web::handler::api::system::docs::tag_status;
use lsys_web::handler::api::system::docs::GitAddParam;
use lsys_web::handler::api::system::docs::GitDelParam;
use lsys_web::handler::api::system::docs::GitDetailParam;
use lsys_web::handler::api::system::docs::GitEditParam;
use lsys_web::handler::api::system::docs::MenuAddParam;
use lsys_web::handler::api::system::docs::MenuDelParam;
use lsys_web::handler::api::system::docs::MenuListParam;
use lsys_web::handler::api::system::docs::TagAddParam;
use lsys_web::handler::api::system::docs::TagCLoneDelParam;
use lsys_web::handler::api::system::docs::TagDelParam;
use lsys_web::handler::api::system::docs::TagDirParam;
use lsys_web::handler::api::system::docs::TagFileDataParam;
use lsys_web::handler::api::system::docs::TagListParam;
use lsys_web::handler::api::system::docs::TagLogsParam;
use lsys_web::handler::api::system::docs::TagStatusParam;
use tracing::debug;
#[get("/raw/{id}/{path}")]
async fn docs_raw(
    req_dao: Data<ReqQuery>,
    info: web::Path<(u32, String)>,
) -> Result<CustomizeResponder<NamedFile>> {
    let info = info.into_inner();
    let path = file_path(
        &RawReadParam {
            menu_id: info.0,
            url: info.1.to_owned(),
        },
        &req_dao,
    )
    .await
    .map_err(|e| InternalError::new(req_dao.fluent_error_string(&e.into()), StatusCode::OK))?;
    debug!("read raw file:{}", &path.file_path.to_string_lossy());
    let file = NamedFile::open_async(path.file_path).await?;
    let ftype = file.content_type().to_string();
    let mut res = file.customize().insert_header(("x-version", path.version));
    if !ftype.contains("charset") {
        res = res.insert_header(("Content-Type", format!("{};charset=utf-8", ftype)));
    }
    Ok(res)
}

#[post("/setting/{type}")]
pub async fn docs_setting(
    path: actix_web::web::Path<String>,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = match path.into_inner().as_str() {
        "git_add" => git_add(&json_param.param::<GitAddParam>()?, &auth_dao).await,
        "git_edit" => git_edit(&json_param.param::<GitEditParam>()?, &auth_dao).await,
        "git_del" => git_del(&json_param.param::<GitDelParam>()?, &auth_dao).await,
        "git_list" => git_list(&auth_dao).await,
        "git_detail" => git_detail(&json_param.param::<GitDetailParam>()?, &auth_dao).await,
        "tag_add" => tag_add(&json_param.param::<TagAddParam>()?, &auth_dao).await,
        "tag_del" => tag_del(&json_param.param::<TagDelParam>()?, &auth_dao).await,
        "tag_list" => tag_list(&json_param.param::<TagListParam>()?, &auth_dao).await,
        "tag_clone_del" => tag_clone_del(&json_param.param::<TagCLoneDelParam>()?, &auth_dao).await,
        "tag_status" => tag_status(&json_param.param::<TagStatusParam>()?, &auth_dao).await,
        "tag_dir" => tag_dir(&json_param.param::<TagDirParam>()?, &auth_dao).await,
        "tag_logs" => tag_logs(&json_param.param::<TagLogsParam>()?, &auth_dao).await,
        "tag_file_data" => tag_file_info(&json_param.param::<TagFileDataParam>()?, &auth_dao).await,
        "menu_add" => menu_add(&json_param.param::<MenuAddParam>()?, &auth_dao).await,
        "menu_list" => menu_list(&json_param.param::<MenuListParam>()?, &auth_dao).await,
        "menu_del" => menu_del(&json_param.param::<MenuDelParam>()?, &auth_dao).await,
        name => handler_not_found!(name),
    };
    Ok(res.map_err(|e| auth_dao.fluent_error_json_data(&e))?.into())
}

#[post("/read/{type}")]
pub async fn docs_read(
    path: actix_web::web::Path<String>,
    req_dao: ReqQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        "menu" => menu_data(&req_dao).await,
        "md" => md_read(&json_param.param::<MdReadParam>()?, &req_dao).await,
        name => handler_not_found!(name),
    };
    Ok(res.map_err(|e| req_dao.fluent_error_json_data(&e))?.into())
}
