use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessSystemMobileConfirm, AccessUserMobileEdit, AccessUserMobileView},
    {CaptchaParam, JsonData, JsonResult},
};
use lsys_core::fluent_message;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use lsys_user::model::UserMobileStatus;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct MobileAddParam {
    pub area_code: String,
    pub mobile: String,
    pub code: Option<String>,
}
pub async fn user_mobile_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MobileAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserMobileEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mut status = UserMobileStatus::Init;
    if let Some(code) = param.code {
        req_dao
            .web_dao
            .user
            .user_dao
            .user_account
            .user_mobile
            .valid_code_check(&code, &param.area_code, &param.mobile)
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        status = UserMobileStatus::Valid;
    }
    let mobile_id = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .add_mobile(
            &user,
            param.area_code,
            param.mobile,
            status,
            None,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": mobile_id })))
}

#[derive(Debug, Deserialize)]
pub struct MobileSendCodeParam {
    pub area_code: String,
    pub mobile: String,
    pub captcha: CaptchaParam,
}
pub async fn user_mobile_send_code<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MobileSendCodeParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::AddSmsCode);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let mobile_res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_last_mobile(param.area_code.clone(), param.mobile.clone())
        .await;
    match mobile_res {
        Ok(mobile) => {
            if UserMobileStatus::Valid.eq(mobile.status) {
                if mobile.user_id != req_auth.user_data().user_id {
                    return Ok(
                        req_dao.fluent_json_data(fluent_message!("mobile-bind-other-user",{
                            "id": mobile.user_id
                        })), //     JsonData::message(format!(
                             //     "other user bind[{}]",

                             // )
                    );
                } else {
                    return Ok(req_dao.fluent_json_data(fluent_message!("mobile-is-bind")));
                }
            }
        }
        Err(err) => {
            if !err.is_not_found() {
                return Err(req_dao.fluent_json_data(err));
            }
        }
    };
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserMobileEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let (code, ttl) = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .valid_code_set(
            &mut req_dao
                .web_dao
                .user
                .user_dao
                .user_account
                .user_mobile
                .valid_code_builder(),
            &param.area_code,
            &param.mobile,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .sender_smser
        .send_valid_code(
            &param.area_code,
            &param.mobile,
            &code,
            &ttl,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "ttl": ttl })))
}

#[derive(Debug, Deserialize)]
pub struct MobileConfirmParam {
    pub mobile_id: u64,
    pub code: String,
}
pub async fn user_mobile_confirm<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MobileConfirmParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let mobile = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_id(&param.mobile_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    if UserMobileStatus::Delete.eq(mobile.status) {
        return Ok(req_dao.fluent_json_data(fluent_message!("mobile-bad-status")));
    }
    if UserMobileStatus::Init.eq(mobile.status) {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(&AccessSystemMobileConfirm {}, None)
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        req_dao
            .web_dao
            .user
            .user_dao
            .user_account
            .user_mobile
            .confirm_mobile_from_code(&mobile, &param.code, Some(&req_dao.req_env))
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MobileDeleteParam {
    pub mobile_id: u64,
}
pub async fn user_mobile_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MobileDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_id(&param.mobile_id)
        .await;

    match res {
        Ok(mobile) => {
            if UserMobileStatus::Init.eq(mobile.status) || UserMobileStatus::Valid.eq(mobile.status)
            {
                req_dao
                    .web_dao
                    .user
                    .rbac_dao
                    .rbac
                    .check(
                        &AccessUserMobileEdit {
                            user_id: req_auth.user_data().user_id,
                            res_user_id: mobile.user_id,
                        },
                        None,
                    )
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_mobile
                    .del_mobile(&mobile, None, Some(&req_dao.req_env))
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
            }
        }
        Err(e) => {
            if !e.is_not_found() {
                return Err(req_dao.fluent_json_data(e));
            }
        }
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct MobileListDataParam {
    pub status: Option<Vec<i8>>,
}
pub async fn user_mobile_list_data<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MobileListDataParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserMobileView {
                user_id: req_auth.user_data().user_id,
                res_user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let status = if let Some(e) = param.status {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match UserMobileStatus::try_from(tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(req_dao.fluent_json_data(err)),
            };
        }
        Some(out)
    } else {
        None
    };
    let data = req_dao
        .web_dao
        .user
        .user_mobile(req_auth.user_data().user_id, status.as_deref())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({
        "data": data ,
        "total":data.len(),
    })))
}
