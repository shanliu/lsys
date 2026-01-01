use actix_files::NamedFile;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::web::Data;
use actix_web::Either;
use actix_web::{http::StatusCode, Result};
use actix_web::{HttpResponse, Responder};
use lsys_web::dao::WebDao;

pub(crate) async fn render_404(app: Data<WebDao>) -> impl Responder {
    if let Ok(file) = app.app_core.config.find(None).get_string("ui_404_file") {
        if let Ok(t) = app
            .app_core
            .config_path(app.app_core.config.find(None), "ui_dir")
        {
            let out = t.join(file);
            if out.is_file() {
                if let Ok(file) = NamedFile::open_async(out).await {
                    return Either::Left(file.customize());
                }
            }
        }
    };
    Either::Right(HttpResponse::build(StatusCode::NOT_FOUND).body("404 - File not found"))
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
