use crate::common::handler::{
    BearerQuery, JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::post;
use actix_web::web::Data;
use lsys_web::dao::WebDao;

use lsys_web::handler::api::system::file::AdminFileDeleteParam;
use lsys_web::handler::api::system::file::AdminFileDownloadingListParam;
use lsys_web::handler::api::system::file::AdminFileLineageRelatedListParam;
use lsys_web::handler::api::system::file::AdminFileListParam;
use lsys_web::handler::api::system::file::AdminOssConfigAddParam;
use lsys_web::handler::api::system::file::AdminOssConfigDeleteParam;
use lsys_web::handler::api::system::file::AdminOssConfigDetailParam;
use lsys_web::handler::api::system::file::AdminOssConfigEditParam;
use lsys_web::handler::api::system::file::AdminOssConfigListParam;
use lsys_web::handler::api::system::file::AdminRuntimeSettingUpdateParam;
use lsys_web::handler::api::system::file::admin_file_delete;
use lsys_web::handler::api::system::file::admin_file_downloading_list;
use lsys_web::handler::api::system::file::admin_file_lineage_related_list;
use lsys_web::handler::api::system::file::admin_file_list;
use lsys_web::handler::api::system::file::admin_file_mapping;
use lsys_web::handler::api::system::file::admin_oss_config_add;
use lsys_web::handler::api::system::file::admin_oss_config_delete;
use lsys_web::handler::api::system::file::admin_oss_config_detail;
use lsys_web::handler::api::system::file::admin_oss_config_edit;
use lsys_web::handler::api::system::file::admin_oss_config_list;
use lsys_web::handler::api::system::file::admin_runtime_setting_get;
use lsys_web::handler::api::system::file::admin_runtime_setting_update;

#[post("/{type}")]
pub async fn file(
    auth_dao: UserAuthQuery,
    path: actix_web::web::Path<String>,
    json_param: JsonQuery,
    bearer: BearerQuery,
    req_query: ReqQuery,
    web_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao
        .set_request_token(&bearer)
        .await
        .map_err(|e| req_query.fluent_error_json_response(&e))?;
    Ok(match path.into_inner().as_str() {
        "mapping" => admin_file_mapping(&req_query, &auth_dao, web_dao.as_ref()).await,
        "list" => {
            admin_file_list(
                &json_param.param::<AdminFileListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "lineage_related_list" => {
            admin_file_lineage_related_list(
                &json_param.param::<AdminFileLineageRelatedListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "downloading_list" => {
            admin_file_downloading_list(
                &json_param.param::<AdminFileDownloadingListParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "delete" => {
            admin_file_delete(&json_param.param::<AdminFileDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oss_config_list" => {
            admin_oss_config_list(&json_param.param::<AdminOssConfigListParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oss_config_detail" => {
            admin_oss_config_detail(&json_param.param::<AdminOssConfigDetailParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "oss_config_add" => {
            admin_oss_config_add(&json_param.param::<AdminOssConfigAddParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oss_config_edit" => {
            admin_oss_config_edit(&json_param.param::<AdminOssConfigEditParam>()?, &req_query, &auth_dao, web_dao.as_ref()).await
        }
        "oss_config_delete" => {
            admin_oss_config_delete(&json_param.param::<AdminOssConfigDeleteParam>()?, &req_query, &auth_dao, web_dao.as_ref())
                .await
        }
        "runtime_setting_get" => admin_runtime_setting_get(&req_query, &auth_dao, web_dao.as_ref()).await,
        "runtime_setting_update" => {
            admin_runtime_setting_update(
                &json_param.param::<AdminRuntimeSettingUpdateParam>()?,
                &req_query,
                &auth_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_query.fluent_error_json_response(&e))?
    .into())
}
