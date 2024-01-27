mod app;
mod oauth;
use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web::scope, App, Error};
use lsys_app::model::AppsModel;

use crate::common::handler::{ResponseJsonResult, RestQuery};

impl RestQuery {
    pub async fn to_app_model(&self) -> ResponseJsonResult<AppsModel> {
        Ok(self
            .web_dao
            .app
            .app_dao
            .app
            .find_by_client_id(&self.rfc.app_id)
            .await
            .map_err(|err| self.fluent_json_data(err))?)
    }
}
pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.service(
        scope("/rest")
            .service(app::access)
            .service(app::app)
            .service(app::sms)
            .service(app::mail)
            .service(app::demo_app),
    )
    .service(
        scope("/oauth")
            .service(oauth::token)
            .service(oauth::refresh)
            .service(oauth::user_data),
    )
}
