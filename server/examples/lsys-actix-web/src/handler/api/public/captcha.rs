use std::str::FromStr;

use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, HttpResponse};
use lsys_web::dao::CaptchaKey;

use crate::common::handler::ReqQuery;

#[derive(Debug, serde::Deserialize)]
pub struct CaptchaParam {
    pub captcha_type: String,
    pub captcha_tag: String,
}

#[get("/{captcha_type}/{captcha_tag}")]
pub(crate) async fn captcha(
    param: actix_web::web::Path<CaptchaParam>,
    req_dao: ReqQuery,
) -> HttpResponse {
    match CaptchaKey::from_str(param.captcha_type.as_str()) {
        Ok(captcha_key) => {
            let valid_code = req_dao.web_dao.app_captcha.valid_code(&captcha_key);
            let mut valid_code_data = req_dao.web_dao.app_captcha.valid_code_builder();
            match valid_code
                .set_code(&param.captcha_tag, &mut valid_code_data)
                .await
            {
                Ok(_) => HttpResponse::Ok()
                    .content_type(valid_code_data.image_header)
                    .append_header(CacheControl(vec![
                        CacheDirective::Private,
                        CacheDirective::MaxAge(valid_code_data.save_time as u32),
                    ]))
                    .body(valid_code_data.image_data),
                Err(err) => HttpResponse::InternalServerError()
                    .body(req_dao.fluent_error_string(&err.into())),
            }
        }
        Err(_) => HttpResponse::NotFound().body("not find"),
    }
}
