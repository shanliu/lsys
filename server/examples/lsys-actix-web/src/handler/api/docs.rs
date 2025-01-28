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
use lsys_web::handler::api::public::{
    docs_file, docs_md_read, docs_menu, DocsMdReadParam, DocsRawReadParam,
};
use lsys_web::handler::api::system::docs_git_add;
use lsys_web::handler::api::system::docs_git_del;
use lsys_web::handler::api::system::docs_git_detail;
use lsys_web::handler::api::system::docs_git_edit;
use lsys_web::handler::api::system::docs_git_list;
use lsys_web::handler::api::system::docs_menu_add;
use lsys_web::handler::api::system::docs_menu_del;
use lsys_web::handler::api::system::docs_menu_list;
use lsys_web::handler::api::system::docs_tag_add;
use lsys_web::handler::api::system::docs_tag_clone_del;
use lsys_web::handler::api::system::docs_tag_del;
use lsys_web::handler::api::system::docs_tag_dir;
use lsys_web::handler::api::system::docs_tag_file_info;
use lsys_web::handler::api::system::docs_tag_list;
use lsys_web::handler::api::system::docs_tag_logs;
use lsys_web::handler::api::system::docs_tag_status;
use lsys_web::handler::api::system::DocsGitAddParam;
use lsys_web::handler::api::system::DocsGitDelParam;
use lsys_web::handler::api::system::DocsGitDetailParam;
use lsys_web::handler::api::system::DocsGitEditParam;
use lsys_web::handler::api::system::DocsMenuAddParam;
use lsys_web::handler::api::system::DocsMenuDelParam;
use lsys_web::handler::api::system::DocsMenuListParam;
use lsys_web::handler::api::system::DocsTagAddParam;
use lsys_web::handler::api::system::DocsTagCLoneDelParam;
use lsys_web::handler::api::system::DocsTagDelParam;
use lsys_web::handler::api::system::DocsTagDirParam;
use lsys_web::handler::api::system::DocsTagFileDataParam;
use lsys_web::handler::api::system::DocsTagListParam;
use lsys_web::handler::api::system::DocsTagLogsParam;
use lsys_web::handler::api::system::DocsTagStatusParam;
use tracing::debug;
#[get("/raw/{id}/{path}")]
async fn docs_raw(
    req_dao: Data<ReqQuery>,
    info: web::Path<(u32, String)>,
) -> Result<CustomizeResponder<NamedFile>> {
    let info = info.into_inner();
    let path = docs_file(
        &DocsRawReadParam {
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
        "git_add" => docs_git_add(&json_param.param::<DocsGitAddParam>()?, &auth_dao).await,
        "git_edit" => docs_git_edit(&json_param.param::<DocsGitEditParam>()?, &auth_dao).await,
        "git_del" => docs_git_del(&json_param.param::<DocsGitDelParam>()?, &auth_dao).await,
        "git_list" => docs_git_list(&auth_dao).await,
        "git_detail" => {
            docs_git_detail(&json_param.param::<DocsGitDetailParam>()?, &auth_dao).await
        }
        "tag_add" => docs_tag_add(&json_param.param::<DocsTagAddParam>()?, &auth_dao).await,
        "tag_del" => docs_tag_del(&json_param.param::<DocsTagDelParam>()?, &auth_dao).await,
        "tag_list" => docs_tag_list(&json_param.param::<DocsTagListParam>()?, &auth_dao).await,
        "tag_clone_del" => {
            docs_tag_clone_del(&json_param.param::<DocsTagCLoneDelParam>()?, &auth_dao).await
        }
        "tag_status" => {
            docs_tag_status(&json_param.param::<DocsTagStatusParam>()?, &auth_dao).await
        }
        "tag_dir" => docs_tag_dir(&json_param.param::<DocsTagDirParam>()?, &auth_dao).await,
        "tag_logs" => docs_tag_logs(&json_param.param::<DocsTagLogsParam>()?, &auth_dao).await,
        "tag_file_data" => {
            docs_tag_file_info(&json_param.param::<DocsTagFileDataParam>()?, &auth_dao).await
        }
        "menu_add" => docs_menu_add(&json_param.param::<DocsMenuAddParam>()?, &auth_dao).await,
        "menu_list" => docs_menu_list(&json_param.param::<DocsMenuListParam>()?, &auth_dao).await,
        "menu_del" => docs_menu_del(&json_param.param::<DocsMenuDelParam>()?, &auth_dao).await,
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
        "menu" => docs_menu(&req_dao).await,
        "md" => docs_md_read(&json_param.param::<DocsMdReadParam>()?, &req_dao).await,
        name => handler_not_found!(name),
    };
    Ok(res.map_err(|e| req_dao.fluent_error_json_data(&e))?.into())
}
