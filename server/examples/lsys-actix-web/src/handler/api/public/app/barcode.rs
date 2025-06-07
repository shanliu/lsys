use crate::common::handler::ReqQuery;
use actix_web::{get, HttpResponse};
use lsys_web::{
    common::JsonError,
    handler::api::public::app::{barcode_show, BarCodeShowCodeParam},
};

#[get("/{content_type}/{code_id}/{content_data:.*}", name = "barcode_show")]
pub(crate) async fn show_code(
    param: actix_web::web::Path<BarCodeShowCodeParam>,
    req_dao: ReqQuery,
) -> HttpResponse {
    match barcode_show(&param, &req_dao).await {
        Ok(img) => HttpResponse::Ok()
            .content_type(img.0.to_mime_type())
            .body(img.1),
        Err(ref err) => match err {
            JsonError::Error(json_err) => {
                if json_err.to_json_data(&req_dao.fluent).code.as_str() == "404" {
                    return HttpResponse::NotFound().body(req_dao.fluent_error_string(err));
                }
                HttpResponse::InternalServerError().body(req_dao.fluent_error_string(err))
            }
            err => HttpResponse::InternalServerError().body(req_dao.fluent_error_string(err)),
        },
    }
}
