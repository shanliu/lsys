use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;

use lsys_web::handler::api::system::export_task::admin_export_active_count;
use lsys_web::handler::api::system::export_task::admin_export_list;
use lsys_web::handler::api::system::export_task::admin_export_submit;
use lsys_web::handler::api::system::export_task::admin_export_task_mapping;
use lsys_web::handler::api::system::export_task::AdminExportActiveCountParam;
use lsys_web::handler::api::system::export_task::AdminExportListParam;
use lsys_web::handler::api::system::export_task::AdminExportSubmitParam;
use lsys_web::handler::api::system::file::admin_file_delete;
use lsys_web::handler::api::system::file::admin_file_list;
use lsys_web::handler::api::system::file::admin_file_mapping;
use lsys_web::handler::api::system::file::admin_oss_config_add;
use lsys_web::handler::api::system::file::admin_oss_config_delete;
use lsys_web::handler::api::system::file::admin_oss_config_detail;
use lsys_web::handler::api::system::file::admin_oss_config_edit;
use lsys_web::handler::api::system::file::admin_oss_config_list;
use lsys_web::handler::api::system::file::AdminFileDeleteParam;
use lsys_web::handler::api::system::file::AdminFileListParam;
use lsys_web::handler::api::system::file::AdminOssConfigAddParam;
use lsys_web::handler::api::system::file::AdminOssConfigDeleteParam;
use lsys_web::handler::api::system::file::AdminOssConfigDetailParam;
use lsys_web::handler::api::system::file::AdminOssConfigEditParam;
use lsys_web::handler::api::system::file::AdminOssConfigListParam;

#[post("/{type}")]
pub async fn file(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&jwt)
        .await
        .map_err(|e| auth_dao.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => admin_file_mapping(&auth_dao).await,
        "list" => admin_file_list(&json_param.param::<AdminFileListParam>()?, &auth_dao).await,
        "delete" => {
            admin_file_delete(&json_param.param::<AdminFileDeleteParam>()?, &auth_dao).await
        }
        "export_task_mapping" => admin_export_task_mapping(&auth_dao).await,
        "export_active_count" => {
            admin_export_active_count(
                &json_param.param::<AdminExportActiveCountParam>()?,
                &auth_dao,
            )
            .await
        }
        "export_submit" => {
            admin_export_submit(
                &json_param.param::<AdminExportSubmitParam>()?,
                &auth_dao,
            )
            .await
        }
        "export_list" => {
            admin_export_list(&json_param.param::<AdminExportListParam>()?, &auth_dao).await
        }
        "oss_config_list" => {
            admin_oss_config_list(
                &json_param.param::<AdminOssConfigListParam>()?,
                &auth_dao,
            )
            .await
        }
        "oss_config_detail" => {
            admin_oss_config_detail(
                &json_param.param::<AdminOssConfigDetailParam>()?,
                &auth_dao,
            )
            .await
        }
        "oss_config_add" => {
            admin_oss_config_add(
                &json_param.param::<AdminOssConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }
        "oss_config_edit" => {
            admin_oss_config_edit(
                &json_param.param::<AdminOssConfigEditParam>()?,
                &auth_dao,
            )
            .await
        }
        "oss_config_delete" => {
            admin_oss_config_delete(
                &json_param.param::<AdminOssConfigDeleteParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}

