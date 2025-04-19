use std::collections::HashMap;

use actix_http::StatusCode;
use actix_web::web::{Bytes, Data};
use actix_web::{post, HttpRequest, HttpResponse};

use lsys_app_sender::dao::{AliYunNotify, CloOpenNotify, HwYunNotify, NetEaseNotify, TenYunNotify};
use lsys_core::IntoFluentMessage;
use lsys_web::dao::WebDao;

#[post("sms/{config_id}/{callback_key}", name = "sms_notify")]
pub(crate) async fn notify(
    path: actix_web::web::Path<(u64, Option<String>)>,
    web_dao: Data<WebDao>,
    body: Bytes,
    req: HttpRequest,
) -> HttpResponse {
    let config = match web_dao
        .web_setting
        .setting_dao
        .multiple
        .find(None, path.0)
        .await
    {
        Ok(e) => e,
        Err(e) => return HttpResponse::Forbidden().body(e.to_fluent_message().default_format()),
    };
    let notify = &web_dao.app_sender.smser.smser_dao.sms_notify;
    let (status, msg) = if notify.check::<AliYunNotify>(&config) {
        let notify_body = String::from_utf8_lossy(&body).to_string();
        let notify_data = AliYunNotify::new(path.1.as_deref().unwrap_or_default(), &notify_body);
        notify.output::<AliYunNotify>(&notify.save(config, notify_data).await)
    } else if notify.check::<CloOpenNotify>(&config) {
        let notify_body = String::from_utf8_lossy(&body).to_string();
        let notify_data = CloOpenNotify::new(path.1.as_deref().unwrap_or_default(), &notify_body);
        notify.output::<CloOpenNotify>(&notify.save(config, notify_data).await)
    } else if notify.check::<HwYunNotify>(&config) {
        let hashmap = match serde_urlencoded::from_bytes::<HashMap<String, String>>(&body) {
            Ok(hashmap) => hashmap,
            Err(err) => {
                return HttpResponse::BadRequest().body(format!("bad from req:{}", err));
            }
        };
        let notify_data = HwYunNotify::new(path.1.as_deref().unwrap_or_default(), &hashmap);
        notify.output::<HwYunNotify>(&notify.save(config, notify_data).await)
    } else if notify.check::<NetEaseNotify>(&config) {
        let notify_body = String::from_utf8_lossy(&body).to_string();
        let headers = req.headers();
        if let (Some(md5), Some(cur_time), Some(check_sum)) = (
            headers.get("MD5"),
            headers.get("CurTime"),
            headers.get("CheckSum"),
        ) {
            if let (Ok(md5), Ok(cur_time), Ok(check_sum)) =
                (md5.to_str(), cur_time.to_str(), check_sum.to_str())
            {
                let notify_data =
                    NetEaseNotify::new(&notify_body, Some((md5, cur_time, check_sum)));
                notify.output::<NetEaseNotify>(&notify.save(config, notify_data).await)
            } else {
                return HttpResponse::BadRequest().body("parse header wrong");
            }
        } else {
            return HttpResponse::BadRequest().body("miss header");
        }
    } else if notify.check::<TenYunNotify>(&config) {
        let notify_body = String::from_utf8_lossy(&body).to_string();
        let notify_data = TenYunNotify::new(path.1.as_deref().unwrap_or_default(), &notify_body);
        notify.output::<TenYunNotify>(&notify.save(config, notify_data).await)
    } else {
        return HttpResponse::NotFound().body("unkown key type");
    };
    HttpResponse::Ok()
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::OK))
        .body(msg)
}
