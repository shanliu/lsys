mod auth;
mod notify;
mod public;
mod system;
mod user;

//API 接口:每个方法路径为子路径,外层路径由scope定
use actix_service::ServiceFactory;
use actix_web::{App, Error, dev::ServiceRequest, web::scope};
pub(crate) fn router<T>(mut app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app = app
        .service(scope("/notify").service(notify::sms::notify))
        .service(scope("/captcha").service(public::captcha));
    let mut api_scope = scope("/api");

    api_scope = api_scope
        .service(scope("/captcha").service(public::captcha_json))
        .service(
            scope("/auth")
                .service(auth::perm)
                .service(auth::login)
                .service(auth::logout)
                .service(auth::user_data)
                .service(auth::external_login_url)
                .service(auth::external_state_callback)
                .service(auth::external_state_check)
                .service(auth::password)
                .service(auth::register),
        )
        .service(scope("/oauth").service(auth::oauth))
        .service(scope("/site").service(public::site_info))
        .service(scope("/area").service(public::area_data))
        .service(scope("/file").service(public::upload_by_token));

    let mut system_scope = scope("/system");
    system_scope = system_scope
        .service(scope("/user").service(system::user))
        .service(
            scope("/config")
                .service(system::site_config)
                .service(system::oauth_config),
        )
        .service(scope("/app").service(system::app))
        .service(
            scope("/sender")
                .service(system::app_sender::mailer)
                .service(system::app_sender::smser),
        )
        .service(
            scope("/rbac")
                .service(system::rbac::base)
                .service(system::rbac::op)
                .service(system::rbac::res)
                .service(system::rbac::role),
        )
        .service(
            scope("/file")
                .service(scope("/collector").service(system::collector))
                .service(system::file),
        );
    api_scope = api_scope.service(system_scope);
    let mut user_scope = scope("/user");
    user_scope = user_scope
        .service(
            scope("/profile")
                .service(user::profile::address)
                .service(user::profile::email)
                .service(user::profile::mobile)
                .service(user::profile::external),
        )
        .service(scope("/base").service(user::base))
        .service(scope("/mfa").service(user::mfa))
        .service(
            scope("/rbac")
                .service(user::rbac::base)
                .service(user::rbac::res)
                .service(user::rbac::role),
        )
        .service(
            scope("/app_rbac")
                .service(user::app::rbac::base)
                .service(user::app::rbac::op)
                .service(user::app::rbac::res)
                .service(user::app::rbac::role),
        )
        .service(
            scope("/app_sender")
                .service(user::app::sender::mailer)
                .service(user::app::sender::smser),
        )
        .service(
            scope("/app_file")
                .service(user::app::file::file_upload_data)
                .service(user::app::file::file),
        )
        .service(scope("/app_export_task").service(user::app::file::export_task))
        .service(scope("/app_collector").service(user::app::file::collector))
        .service(scope("/app").service(user::app::base));
    api_scope = api_scope.service(user_scope);
    app.service(api_scope)
}
