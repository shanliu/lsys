use crate::dao::App;

use crate::model::AppModel;
use crate::model::AppSecretType;
use lsys_core::IntoFluentMessage;
use serde_json::json;

use tracing::info;
use tracing::warn;

pub const SUB_APP_SECRET_NOTIFY_TYPE: &str = "sub_app_notify";

impl App {
    pub(crate) async fn add_app_secret_change_notify(&self, app: &AppModel) {
        if app.parent_app_id == 0 {
            info!("System app Ignore notify:{}", app.id);
            return;
        }
        match self
            .app_secret
            .multiple_find_secret_by_app_id(app.id, AppSecretType::App)
            .await
        {
            Ok(secret) => {
                if let Err(err) = self
                    .app_notify
                    .add(
                        SUB_APP_SECRET_NOTIFY_TYPE,
                        app.parent_app_id,
                        &json!({
                            "client_id":app.client_id,
                            "sercet_data":secret,
                        })
                        .to_string(),
                    )
                    .await
                {
                    warn!(
                        "add notify data fail:{}",
                        err.to_fluent_message().default_format()
                    );
                }
            }
            Err(e) => {
                warn!(
                    "get app secret fail:{}",
                    e.to_fluent_message().default_format()
                );
            }
        }
    }
}
