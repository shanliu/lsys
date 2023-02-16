use crate::{dao::WebDao, JsonData, JsonResult};
use lsys_app::model::AppsModel;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SmsSendParam {
    pub mobile: String,
    pub tpl: String,
    pub data: String,
    pub cancel: Option<String>,
}
pub async fn sms_send(
    app_dao: &WebDao,
    app: &AppsModel,
    param: SmsSendParam,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            app.user_id,
            &[app_dao.app.app_relation_key(app).await],
            &res_data!(AppSender(app.id, app.user_id)),
        )
        .await?;
    app_dao
        .smser
        .app_send(
            app,
            &param.tpl,
            "86",
            &param.mobile,
            &param.data,
            None,
            &param.cancel,
        )
        .await?;
    Ok(JsonData::message("success").set_data(json!({ "pass": 1 })))
}

#[derive(Debug, Deserialize)]
pub struct SmsCancelParam {
    pub cancel: String,
}
pub async fn sms_cancel(
    app_dao: &WebDao,
    app: &AppsModel,
    param: SmsCancelParam,
) -> JsonResult<JsonData> {
    app_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            app.user_id,
            &[app_dao.app.app_relation_key(app).await],
            &res_data!(AppSender(app.id, app.user_id)),
        )
        .await?;
    app_dao.smser.app_send_cancel(app, &param.cancel).await?;
    Ok(JsonData::message("success").set_data(json!({ "pass": 1 })))
}
