use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use actix_web::{post, HttpRequest};
use lsys_web::handler::api::user::app_barcode::{
    create_config_add, create_config_delete, create_config_edit, create_config_list, mapping_data,
    parse_record_delete, parse_record_list, CreateConfigAddParam, CreateConfigDeleteParam,
    CreateConfigEditParam, CreateConfigListParam, ParseRecordDeleteParam, ParseRecordListParam,
};
#[post("/{type}")]
pub async fn barcode(
    path: actix_web::web::Path<String>,
    auth_dao: UserAuthQuery,
    json_param: JsonQuery,
    jwt: JwtQuery,
    req: HttpRequest,
) -> ResponseJsonResult<ResponseJson> {
    auth_dao.set_request_token(&jwt).await;
    Ok(match path.into_inner().as_str() {
        "mapping" => mapping_data(&auth_dao).await,
        "create_config_add" => {
            create_config_add(&json_param.param::<CreateConfigAddParam>()?, &auth_dao).await
        }
        "create_config_edit" => {
            create_config_edit(&json_param.param::<CreateConfigEditParam>()?, &auth_dao).await
        }
        "create_config_list" => {
            create_config_list(
                &json_param.param::<CreateConfigListParam>()?,
                &auth_dao,
                |item| {
                    req.url_for("barcode_show", [item.id.to_string(), "".to_string()])
                        .map(|e| e.to_string())
                        .unwrap_or_default()
                },
            )
            .await
        }
        "create_config_delete" => {
            create_config_delete(&json_param.param::<CreateConfigDeleteParam>()?, &auth_dao).await
        }
        "parse_record_delete" => {
            parse_record_delete(&json_param.param::<ParseRecordDeleteParam>()?, &auth_dao).await
        }
        "parse_record_list" => {
            parse_record_list(&json_param.param::<ParseRecordListParam>()?, &auth_dao).await
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| auth_dao.fluent_error_json_response(&e))?
    .into())
}
