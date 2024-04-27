use crate::common::handler::{
    JsonQuery, JwtQuery, ResponseJson, ResponseJsonResult, UserAuthQuery,
};

use lsys_web::handler::api::barcode::{
    barcode_create_config_add, barcode_create_config_delete, barcode_create_config_edit,
    barcode_create_config_list, barcode_parse_record_delete, barcode_parse_record_list,
    BarCodeCreateConfigAddParam, BarCodeCreateConfigDeleteParam, BarCodeCreateConfigEditParam,
    BarCodeCreateConfigListParam, BarCodeParseRecordDeleteParam, BarCodeParseRecordListParam,
};
use actix_web::{post, HttpRequest};
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
        "create_config_add" => {
            barcode_create_config_add(
                json_param.param::<BarCodeCreateConfigAddParam>()?,
                &auth_dao,
            )
            .await
        }
        "create_config_edit" => {
            barcode_create_config_edit(
                json_param.param::<BarCodeCreateConfigEditParam>()?,
                &auth_dao,
            )
            .await
        }
        "create_config_list" => {
            barcode_create_config_list(
                json_param.param::<BarCodeCreateConfigListParam>()?,
                &auth_dao,
                |item| {
                    req.url_for(
                        "barcode_show",
                        &[item.id.to_string(),"".to_string()],
                    )
                    .map(|e| e.to_string())
                    .unwrap_or_default()
                },
            )
            .await
        }
        "create_config_delete" => {
            barcode_create_config_delete(
                json_param.param::<BarCodeCreateConfigDeleteParam>()?,
                &auth_dao,
            )
            .await
        }
        "parse_record_delete" => {
            barcode_parse_record_delete(
                json_param.param::<BarCodeParseRecordDeleteParam>()?,
                &auth_dao,
            )
            .await
        }
        "parse_record_list" => {
            barcode_parse_record_list(
                json_param.param::<BarCodeParseRecordListParam>()?,
                &auth_dao,
            )
            .await
        }
        name => handler_not_found!(name),
    }?
    .into())
}
