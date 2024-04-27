use actix_web::{get, HttpResponse};
use lsys_web::handler::api::barcode::{barcode_show, BarCodeShowParam};

use crate::common::handler::ReqQuery;

#[get("/{code_id}/{contents:.*}", name = "barcode_show")]
pub(crate) async fn show_code(
    param: actix_web::web::Path<BarCodeShowParam>,
    req_dao: ReqQuery,
) -> HttpResponse {
    match barcode_show(&param, &req_dao,true).await {
        Ok(img) => HttpResponse::Ok()
            .content_type(img.0.to_mime_type())
            .body(img.1),
        Err(err) => HttpResponse::InternalServerError().body(err.to_value().to_string()),
    }
}
