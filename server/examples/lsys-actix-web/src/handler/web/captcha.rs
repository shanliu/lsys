use std::str::FromStr;

use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, HttpResponse};
use lsys_web::dao::CaptchaKey;

use crate::common::handler::ReqQuery;

#[get("/{type}/{tag}")]
pub(crate) async fn captcha(
    path: actix_web::web::Path<(String, String)>,
    req_dao: ReqQuery,
) -> HttpResponse {
    let path = path.into_inner();
    match CaptchaKey::from_str(path.0.as_str()) {
        Ok(captcha_key) => {
            let valid_code = req_dao.web_dao.captcha.valid_code(&captcha_key);
            let mut valid_code_data = req_dao.web_dao.captcha.valid_code_builder();
            match valid_code.set_code(&path.1, &mut valid_code_data).await {
                Ok(_) => HttpResponse::Ok()
                    .content_type(valid_code_data.image_header)
                    .append_header(CacheControl(vec![
                        CacheDirective::Private,
                        CacheDirective::MaxAge(valid_code_data.save_time as u32),
                    ]))
                    .body(valid_code_data.image_data),
                Err(err) => HttpResponse::InternalServerError().body(req_dao.fluent_string(err)),
            }
        }
        Err(_) => HttpResponse::NotFound().body("not find"),
    }
}
