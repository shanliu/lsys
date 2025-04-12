mod site_setting;
use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::CheckAdminSiteSetting;
use lsys_access::dao::AccessSession;
use lsys_setting::dao::NotFoundResult;
use lsys_setting::dao::SingleSettingData;
use lsys_setting::dao::{SettingDecode, SettingEncode, SettingKey};
use serde::{Deserialize, Serialize};
use serde_json::json;
pub use site_setting::*;

pub async fn setting_set<
    'a,
    P: Deserialize<'a>,
    A: SettingKey + SettingDecode + SettingEncode + From<P>,
>(
    param: P,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAdminSiteSetting {},
        )
        .await?;
    req_dao
        .web_dao
        .web_setting
        .setting_dao
        .single
        .save::<A>(
            None,
            &SingleSettingData {
                name: A::key(),
                data: &A::from(param),
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

pub async fn setting_get<A: SettingKey + SettingDecode + SettingEncode + Serialize + Default>(
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckAdminSiteSetting {},
        )
        .await?;
    let data = req_dao
        .web_dao
        .web_setting
        .setting_dao
        .single
        .load::<A>(None)
        .await
        .notfound_default()?;
    Ok(JsonResponse::data(JsonData::body(
        json!({ "config":  &*data }),
    )))
}
