use actix_web::{get, HttpResponse};
use lsys_web::handler::api::public::{app_barcode_show, BarCodeShowParam};

use crate::common::handler::ReqQuery;

#[get("/{code_id}/{contents:.*}", name = "barcode_show")]
pub(crate) async fn show_code(
    param: actix_web::web::Path<BarCodeShowParam>,
    req_dao: ReqQuery,
) -> HttpResponse {
    match app_barcode_show(&param, &req_dao).await {
        Ok(img) => HttpResponse::Ok()
            .content_type(img.0.to_mime_type())
            .body(img.1),
        Err(err) => HttpResponse::InternalServerError().body(req_dao.fluent_error_string(&err)),
    }
}
