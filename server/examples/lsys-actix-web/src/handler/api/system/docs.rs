use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;

use lsys_web::handler::api::system::docs::git_add;
use lsys_web::handler::api::system::docs::git_del;
use lsys_web::handler::api::system::docs::git_detail;
use lsys_web::handler::api::system::docs::git_edit;
use lsys_web::handler::api::system::docs::git_list;
use lsys_web::handler::api::system::docs::mapping_data;
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

#[post("/{type}")]
pub async fn setting(
    path: actix_web::web::Path<String>,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    let res = match path.into_inner().as_str() {
        "mapping" => mapping_data(&auth_dao).await,
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
    Ok(res
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?
        .into())
}
