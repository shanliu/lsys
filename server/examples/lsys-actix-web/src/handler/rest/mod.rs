//rest 接口
mod app;
mod oauth;
use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web::scope, App, Error};
use lsys_app::model::AppModel;

use crate::common::handler::{ResponseJsonResult, RestQuery};

impl RestQuery {
    pub async fn get_app(&self) -> ResponseJsonResult<AppModel> {
        Ok(self
            .web_dao
            .web_app
            .app_dao
            .app
            .cache()
            .find_by_client_id(&self.rfc.app_id)
            .await
            .map_err(|e| self.fluent_error_json_data(&e.into()))?)
    }
}
pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.service(
        scope("/rest")
            .service(app::access)
            .service(app::barcode)
            .service(app::subapp)
            .service(app::sms)
            .service(app::mail)
            .service(app::auth)
            .service(app::demo_app),
    )
    .service(
        scope("/oauth")
            .service(oauth::token)
            .service(oauth::refresh)
            .service(oauth::user_data),
    )
}
