//后台页面接口(jwt 接口)
use actix_web::{options, HttpResponse, Responder};

#[options("/{_:.*}")]
pub(crate) async fn options() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Access-Control-Allow-Methods", "POST, GET, OPTIONS"))
        .insert_header((
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization, X-Request-ID, X-Device-ID",
        ))
        .finish()
}
