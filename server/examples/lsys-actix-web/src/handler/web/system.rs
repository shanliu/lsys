use actix_files::NamedFile;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::web::Data;
use actix_web::Responder;
use actix_web::{http::StatusCode, Result};
use lsys_web::dao::WebDao;

pub(crate) async fn render_404(app: Data<WebDao>) -> impl Responder {
    let static_serve_from = app
        .app_core
        .config
        .find(None)
        .get_string("static_file_dir")
        .unwrap_or_else(|_| String::from("./static"))
        + "/404.html";
    NamedFile::open_async(static_serve_from)
        .await
        .customize()
        .with_status(StatusCode::NOT_FOUND)
}
pub(crate) fn render_500<B>(
    mut res: actix_web::dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>> {
    res.response_mut().headers_mut().insert(
        actix_web::http::header::CONTENT_TYPE,
        actix_web::http::header::HeaderValue::from_static("Error"),
    );
    Ok(ErrorHandlerResponse::Response(res.map_into_left_body()))
}
