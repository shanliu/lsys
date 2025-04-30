use crate::common::JsonData;
use crate::common::JsonError;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::user::CheckUserBarCodeEdit;
use crate::dao::access::api::user::CheckUserBarCodeView;
use lsys_access::dao::AccessSession;
use lsys_app_barcode::model::BarcodeCreateModel;
use lsys_app_barcode::model::BarcodeCreateStatus;
use lsys_core::fluent_message;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct CreateConfigAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub barcode_type: String,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub status: i8,
    pub image_format: String,
    #[serde(deserialize_with = "crate::common::deserialize_i32")]
    pub image_width: i32,
    #[serde(deserialize_with = "crate::common::deserialize_i32")]
    pub image_height: i32,
    #[serde(deserialize_with = "crate::common::deserialize_i32")]
    pub margin: i32,
    pub image_color: String,
    pub image_background: String,
}

pub async fn create_config_add(
    param: &CreateConfigAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(&param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_BARCODE])
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserBarCodeEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let status = BarcodeCreateStatus::try_from(param.status)
        .map_err(|e| JsonError::Message(fluent_message!("barcode-add-status-error", e)))?;

    let id = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .add_create_config(
            &auth_data.user_id(),
            &param.app_id,
            &status,
            &param.barcode_type,
            &param.image_format,
            &param.image_width,
            &param.image_height,
            &param.margin,
            &param.image_color,
            &param.image_background,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "id":id,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct CreateConfigEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub barcode_type: String,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub status: i8,
    pub image_format: String,
    #[serde(deserialize_with = "crate::common::deserialize_i32")]
    pub image_width: i32,
    #[serde(deserialize_with = "crate::common::deserialize_i32")]
    pub image_height: i32,
    #[serde(deserialize_with = "crate::common::deserialize_i32")]
    pub margin: i32,
    pub image_color: String,
    pub image_background: String,
}

pub async fn create_config_edit(
    param: &CreateConfigEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .find_by_create_config_id(&param.id)
        .await?;

    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .find_by_id(&data.app_id)
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .cache()
        .exter_feature_check(&app, &[crate::handler::APP_FEATURE_BARCODE])
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserBarCodeEdit {
                res_user_id: data.user_id,
            },
        )
        .await?;

    let status = BarcodeCreateStatus::try_from(param.status)
        .map_err(|e| JsonError::Message(fluent_message!("barcode-add-status-error", e)))?;

    req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .edit_create_config(
            &data,
            &auth_data.user_id(),
            &status,
            &param.barcode_type,
            &param.image_format,
            &param.image_width,
            &param.image_height,
            &param.margin,
            &param.image_color,
            &param.image_background,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct CreateConfigDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

pub async fn create_config_delete(
    param: &CreateConfigDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .find_by_create_config_id(&param.id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserBarCodeEdit {
                res_user_id: data.user_id,
            },
        )
        .await?;
    req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .delete_create_config(&auth_data.user_id(), &data, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct CreateConfigListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub app_id: Option<u64>,
    pub barcode_type: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
pub async fn create_config_list(
    param: &CreateConfigListParam,
    req_dao: &UserAuthQueryDao,
    url_callback: impl Fn(&BarcodeCreateModel) -> String,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserBarCodeView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let data = req_dao
        .web_dao
        .app_barcode
        .barcode_dao
        .list_create_config(
            auth_data.user_id(),
            param.id,
            param.app_id,
            param.barcode_type.as_deref(),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?
        .into_iter()
        .map(|e| {
            let url = url_callback(&e);
            json!({
                "id":e.id,
                "barcode_type":e.barcode_type,
                "app_id":e.app_id,
                "change_time":e.change_time,
                "image_background":e.image_background,
                "image_color":e.image_color,
                "image_format":e.image_format,
                "image_height":e.image_height,
                "image_width":e.image_width,
                "margin":e.margin,
                "status":e.status,
                "url":url,
            })
        })
        .collect::<Vec<_>>();
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .app_barcode
                .barcode_dao
                .count_create_config(
                    auth_data.user_id(),
                    param.id,
                    param.app_id,
                    param.barcode_type.as_deref(),
                )
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": data,"total":count }),
    )))
}
