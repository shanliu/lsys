use crate::{
    dao::RequestDao,
    handler::access::{AccessSystemMobileConfirm, AccessUserMobileEdit, AccessUserMobileView},
    {CaptchaParam, JsonData, JsonResult},
};
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
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessUserMobileEdit {
            user_id: req_auth.user_data().user_id,
            res_user_id: req_auth.user_data().user_id,
        })
        .await?;

    let mut status = UserMobileStatus::Init;
    if let Some(code) = param.code {
        req_dao
            .web_dao
            .user
            .user_dao
            .user_account
            .user_mobile
            .valid_code_check(&code, &param.area_code, &param.mobile)
            .await?;
        status = UserMobileStatus::Valid;
    }
    let mobile_id = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .add_mobile(&user, param.area_code, param.mobile, status, None)
        .await?;
    Ok(JsonData::data(json!({ "id": mobile_id })))
}

#[derive(Debug, Deserialize)]
pub struct MobileSendCodeParam {
    pub area_code: String,
    pub mobile: String,
    pub captcha: CaptchaParam,
}
pub async fn user_mobile_send_code<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MobileSendCodeParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::AddSmsCode);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await?;
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
                    return Ok(JsonData::message(format!(
                        "other user bind[{}]",
                        mobile.user_id
                    )));
                } else {
                    return Ok(JsonData::message("the mobile is bind"));
                }
            }
        }
        Err(err) => {
            if !err.is_not_found() {
                return Err(err.into());
            }
        }
    };
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessUserMobileEdit {
            user_id: req_auth.user_data().user_id,
            res_user_id: req_auth.user_data().user_id,
        })
        .await?;

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
        .await?;
    req_dao
        .web_dao
        .smser
        .send_valid_code(&param.area_code, &param.mobile, &code, &ttl)
        .await?;
    Ok(JsonData::message("sms is send").set_data(json!({ "ttl": ttl })))
}

#[derive(Debug, Deserialize)]
pub struct MobileConfirmParam {
    pub mobile_id: u64,
    pub code: String,
}
pub async fn user_mobile_confirm<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MobileConfirmParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let mobile = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_mobile
        .find_by_id(&param.mobile_id)
        .await?;
    if UserMobileStatus::Delete.eq(mobile.status) {
        return Ok(JsonData::message("mobile not find"));
    }
    if UserMobileStatus::Init.eq(mobile.status) {
        req_dao
            .web_dao
            .user
            .rbac_dao
            .rbac
            .check(&AccessSystemMobileConfirm {})
            .await?;

        req_dao
            .web_dao
            .user
            .user_dao
            .user_account
            .user_mobile
            .confirm_mobile_from_code(&mobile, &param.code)
            .await?;
    }
    Ok(JsonData::message("edit mobile ok"))
}

#[derive(Debug, Deserialize)]
pub struct MobileDeleteParam {
    pub mobile_id: u64,
}
pub async fn user_mobile_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: MobileDeleteParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
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
                    .check(&AccessUserMobileEdit {
                        user_id: req_auth.user_data().user_id,
                        res_user_id: mobile.user_id,
                    })
                    .await?;

                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_mobile
                    .del_mobile(&mobile, None)
                    .await?;
            }
        }
        Err(e) => {
            if !e.is_not_found() {
                return Err(e.into());
            }
        }
    }
    Ok(JsonData::message("del mobile ok"))
}

#[derive(Debug, Deserialize)]
pub struct MobileListDataParam {
    pub status: Option<Vec<i8>>,
}
pub async fn user_mobile_list_data<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MobileListDataParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessUserMobileView {
            user_id: req_auth.user_data().user_id,
            res_user_id: req_auth.user_data().user_id,
        })
        .await?;

    let status = if let Some(e) = param.status {
        let mut out = Vec::with_capacity(e.len());
        for tmp in e {
            match UserMobileStatus::try_from(tmp) {
                Ok(ts) => out.push(ts),
                Err(err) => return Err(JsonData::error(err)),
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
        .await?;
    Ok(JsonData::data(json!({
        "data": data ,
        "total":data.len(),
    })))
}
