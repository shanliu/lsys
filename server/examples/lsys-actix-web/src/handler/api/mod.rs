mod notify;
mod public;
mod system;
mod user;

//API 接口:每个方法路径为子路径,外层路径由scope定
use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, web::scope, App, Error};
pub(crate) fn router<T>(mut app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    app = app
    .service(scope("/notify").service(notify::sms::notify))
    .service(
        scope("/captcha")
            .service(public::captcha)
            .service(public::options),
    );
    let mut api_scope = scope("/api");

    api_scope = api_scope
        .service(
            scope("/auth")
                .service(public::login)
                .service(public::logout)
                .service(public::user_data)
                .service(public::external_login_url)
                .service(public::external_state_callback)
                .service(public::external_state_check)
                .service(public::password)
                .service(public::register)
                .service(public::options),
        )
        .service(
            scope("/oauth")
                .service(public::oauth)
                .service(public::options),
        )
        .service(
            scope("/site")
                .service(public::site_info)
                .service(public::options),
        )
        .service(
            scope("/area")
                .service(public::area_data)
                .service(public::options),
        );

    let mut system_scope = scope("/system");
    #[cfg(feature = "docs")]
    {
        app = {
            system_scope = system_scope.service(
                scope("/docs")
                    .service(system::docs::setting)
                    .service(public::options),
            );
            app.service(
                scope("/docs")
                    .service(public::docs_raw)
                    .service(public::docs_read)
                    .service(public::options),
            )
        };
    }

    #[cfg(feature = "barcode")]
    {
        app = {
            app.service(
                scope("/barcode")
                    .service(public::app::show_code)
                    .service(public::options),
            )
        };
    }
    system_scope = system_scope
        .service(
            scope("/user")
                .service(system::user)
                .service(public::options),
        )
        .service(
            scope("/site")
                .service(system::site_config)
                .service(system::oauth_config)
                .service(public::options),
        )
        .service(scope("/app").service(system::app).service(public::options))
        .service(
            scope("/sender")
                .service(system::app_sender::mailer)
                .service(system::app_sender::smser)
                .service(public::options),
        )
        .service(
            scope("/rbac")
                .service(system::rbac::check)
                .service(system::rbac::op)
                .service(system::rbac::res)
                .service(system::rbac::role)
                .service(public::options),
        )
        .service(public::options);
    api_scope = api_scope.service(system_scope);
    let mut user_scope = scope("/user");
    user_scope = user_scope
        .service(
            scope("/profile")
                .service(user::profile::address)
                .service(user::profile::email)
                .service(user::profile::mobile)
                .service(user::profile::external)
                .service(public::options),
        )
        .service(
            scope("/account")
                .service(user::account)
                .service(public::options),
        )
        .service(
            scope("/rbac")
                .service(user::rbac::audit)
                .service(user::rbac::res)
                .service(user::rbac::role)
                .service(public::options),
        )
        .service(
            scope("/app_rbac")
                .service(user::app::rbac::check)
                .service(user::app::rbac::op)
                .service(user::app::rbac::res)
                .service(user::app::rbac::role)
                .service(public::options),
        )
        .service(
            scope("/app_sender")
                .service(user::app::sender::mailer)
                .service(user::app::sender::smser)
                .service(public::options),
        )
        .service(
            scope("/app_notify")
                .service(user::app::notify)
                .service(public::options),
        )
        .service(
            scope("/app")
                .service(user::app::base)
                .service(public::options),
        );
    #[cfg(feature = "barcode")]
    {
        api_scope = api_scope.service(
            scope("/app_barcode")
                .service(user::app::barcode)
                .service(public::options),
        );
    }
    api_scope = api_scope.service(user_scope).service(public::options);
    app.service(api_scope)
}
