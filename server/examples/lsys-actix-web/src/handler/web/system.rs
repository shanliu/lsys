use actix_files::NamedFile;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::web::Data;
use actix_web::Either;
use actix_web::{http::StatusCode, Result};
use actix_web::{HttpResponse, Responder};
use lsys_web::dao::WebDao;

pub(crate) async fn render_404(app: Data<WebDao>) -> impl Responder {
    let page_404 = match app
        .app_core
        .config_path(app.app_core.config.find(None), "static_file_dir")
    {
        Ok(t) => {
            let out = t.join("404.html");
            if out.is_file() {
                Some(
                    NamedFile::open_async(out)
                        .await
                        .customize()
                        .with_status(StatusCode::NOT_FOUND),
                )
            } else {
                None
            }
        }
        Err(_) => None,
    };
    match page_404 {
        Some(page) => Either::Left(page),
        None => Either::Right(HttpResponse::NotFound().body("404 - File not found")),
    }
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
