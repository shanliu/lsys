use crate::common::JsonData;
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResponse, JsonResult, RequestDao, UserAuthQueryDao},
    dao::{InfoSetUserInfoData, access::api::system::user::CheckUserInfoEdit},
};
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct InfoSetUserNameParam {
    pub name: String,
}
pub async fn info_set_username(
    param: &InfoSetUserNameParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserInfoEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    web_dao
        .web_user
        .account
        .user_info_set_username(&param.name, &auth_data, Some(&req_dao.req_env))
        .await?;
    let token = web_dao
        .web_user
        .user_dao
        .auth_dao
        .reload(auth_dao.user_session.read().await.get_session_token(), false)
        .await?;
    auth_dao.user_session.write().await.set_session_token(token);
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct InfoCheckUserNameParam {
    pub name: String,
}
pub async fn info_check_username(
    param: &InfoCheckUserNameParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserInfoEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    web_dao
        .web_user
        .account
        .user_info_check_username(&param.name)
        .await?;
    Ok(JsonResponse::default().set_data(JsonData::body(json!({
        "pass":"1"
    }))))
}

#[derive(Debug, Deserialize)]
pub struct InfoSetUserInfoParam {
    pub nikename: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i32")]
    pub gender: Option<i32>,
    pub headimg: Option<String>,
    pub birthday: Option<String>,
}
pub async fn info_set_data(
    param: &InfoSetUserInfoParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserInfoEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    // 若 headimg 非空且非 http(s) 开头，则视为文件 key，需校验文件权限
    let headimg_val: Option<String> = match &param.headimg {
        Some(h) if !h.is_empty() && !h.starts_with("http") => {
            let ref_id = web_dao
                .web_file
                .file_dao
                .file_key_encoder()
                .decode(h.as_str())
                .map_err(crate::common::JsonError::from)?;
            let ref_model = web_dao
                .web_file
                .file_dao
                .cache()
                .find_file_ref_by_id(ref_id)
                .await
                .map_err(crate::common::JsonError::from)?;
            if ref_model.add_user_id != auth_data.user_id() {
                return Err(crate::common::JsonError::Message(
                    lsys_core::fluent_message!("file-user-mismatch"),
                ));
            }
            Some(h.clone())
        }
        other => other.clone(),
    };

    web_dao
        .web_user
        .account
        .user_info_set_data(
            &InfoSetUserInfoData {
                nikename: param.nikename.as_deref(),
                gender: param.gender,
                headimg: headimg_val.as_deref(),
                birthday: param.birthday.as_deref(),
            },
            &auth_data,
            Some(&req_dao.req_env),
        )
        .await?;
    let token = web_dao
        .web_user
        .user_dao
        .auth_dao
        .reload(auth_dao.user_session.read().await.get_session_token(), false)
        .await?;
    auth_dao.user_session.write().await.set_session_token(token);

    Ok(JsonResponse::default())
}

pub async fn password_last_modify(
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    let (passwrod, is_expired, timeout_config) = web_dao
        .web_user
        .account
        .password_last_modify(&auth_data)
        .await?;

    // 计算剩余有效时间
    let remaining_time = if timeout_config == 0 {
        0 // 永久有效
    } else {
        let expire_time = passwrod.add_time + timeout_config;
        let current_time = lsys_core::utils::now_time().unwrap_or_default();
        expire_time.saturating_sub(current_time)
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "last_time": passwrod.add_time,
        "remaining_time": remaining_time,
        "is_expired": is_expired,
        "total_timeout": timeout_config,
    }))))
}
