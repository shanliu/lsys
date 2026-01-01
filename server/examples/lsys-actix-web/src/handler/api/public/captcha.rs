use std::str::FromStr;

use actix_web::http::header::{CacheControl, CacheDirective};
use actix_web::{get, post, HttpResponse};
use lsys_web::common::{JsonData, JsonError, JsonResponse};
use lsys_web::dao::CaptchaKey;
use lsys_web::lsys_core::CheckCodeData;
use serde_json::json;

use crate::common::handler::{JsonQuery, ReqQuery, ResponseJson, ResponseJsonResult};

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

#[derive(Debug, serde::Deserialize)]
pub struct CaptchaValidParam {
    pub captcha_type: String,
    pub captcha_tag: String,
    pub captcha_code: String,
}

#[post("/{method}")]
pub(crate) async fn captcha_json(
    req_dao: ReqQuery,
    json_param: JsonQuery,
    path: actix_web::web::Path<String>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match path.into_inner().as_str() {
        "show" => {
            let param = json_param
                .param::<CaptchaParam>()
                .map_err(ResponseJson::from)?;
            let valid_code = req_dao.web_dao.app_captcha.valid_code(
                &CaptchaKey::from_str(&param.captcha_type).map_err(|e| {
                    ResponseJson::from(JsonResponse::data(JsonData::error()).set_message(e))
                })?,
            );
            let mut valid_code_data = req_dao.web_dao.app_captcha.valid_code_builder();

            valid_code
                .set_code(&param.captcha_tag, &mut valid_code_data)
                .await
                .map(|_| {
                    JsonResponse::data(JsonData::body(json!({
                        "image_header":valid_code_data.image_header,
                        "image_data":valid_code_data.base64_image(),
                        "save_time":valid_code_data.save_time,
                        "code_length":valid_code_data.code.len(),
                    })))
                })
                .map_err(JsonError::from)
        }
        "valid" => {
            let param = json_param
                .param::<CaptchaValidParam>()
                .map_err(ResponseJson::from)?;
            let valid_code = req_dao.web_dao.app_captcha.valid_code(
                &CaptchaKey::from_str(&param.captcha_type).map_err(|e| {
                    ResponseJson::from(JsonResponse::data(JsonData::error()).set_message(e))
                })?,
            );
            let out = valid_code
                .check_code(&CheckCodeData {
                    tag: &param.captcha_tag,
                    code: &param.captcha_code,
                })
                .await
                .map(|_| JsonResponse::default())
                .map_err(|e| req_dao.fluent_error_json_response(&e.into()))?;
            return Ok(out.into());
        }
        name => handler_not_found!(name),
    }
    .map_err(|e| req_dao.fluent_error_json_response(&e))?
    .into())
}
