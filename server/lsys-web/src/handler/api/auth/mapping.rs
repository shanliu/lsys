use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::RequestDao;
use lsys_core::FluentMessage;
use lsys_user::dao::login::AccountLoginMeta;
use lsys_user::dao::login::EmailCodeLoginMeta;
use lsys_user::dao::login::EmailLoginMeta;
use lsys_user::dao::login::ExternalLoginMeta;
use lsys_user::dao::login::MobileCodeLoginMeta;
use lsys_user::dao::login::MobileLoginMeta;
use lsys_user::dao::login::NameLoginMeta;
use serde_json::json;
use serde_json::Value;
pub async fn mapping_data(req_dao: &RequestDao, exter: Value) -> JsonResult<JsonResponse> {
    Ok(JsonResponse::data(JsonData::body(json!({
        "exter_type":exter,
        "login_type": ([(
            EmailLoginMeta::login_type(),
            EmailLoginMeta::login_timeout(),
        ),
        (
            EmailCodeLoginMeta::login_type(),
            EmailCodeLoginMeta::login_timeout(),
        ),
        (NameLoginMeta::login_type(), NameLoginMeta::login_timeout()),
        (
            MobileLoginMeta::login_type(),
            MobileLoginMeta::login_timeout(),
        ),
        (
            MobileCodeLoginMeta::login_type(),
            MobileCodeLoginMeta::login_timeout(),
        ),
        (
            ExternalLoginMeta::login_type(),
            ExternalLoginMeta::login_timeout(),
        )]).iter()
    .map(|e| {
        json!({
            "key":&e.0,
            "validity":e.1,
            "val":req_dao.fluent.format_message(&FluentMessage {
                id: format!("dict-login-type-{}",&e.0),
                crate_name:env!("CARGO_PKG_NAME").to_string(),
                data:vec![]
            }),
        })
    })
    .collect::<Vec<_>>()
    }))))
}
