mod oauth;
mod subapp;
use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web::scope, App, Error};
use lsys_app::{dao::app::Apps, model::AppsModel};
use lsys_web::JsonData;

use crate::common::handler::{ResponseJsonResult, RestRfc};

impl RestRfc {
    pub async fn to_app_model(&self, apps: &Apps) -> ResponseJsonResult<AppsModel> {
        apps.find_by_client_id(&self.app)
            .await
            .map_err(|e| JsonData::message(e.to_string()).into())
    }
}
pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app.service(
        scope("/rest")
            .service(subapp::access)
            .service(subapp::app)
            .service(subapp::demo_app),
    )
    .service(
        scope("/oauth")
            .service(oauth::token)
            .service(oauth::refresh)
            .service(oauth::user_data),
    )
}
