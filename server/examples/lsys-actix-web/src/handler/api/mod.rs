use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, options, web::scope, App, Error, HttpResponse, Responder};

mod app;
#[cfg(feature = "area")]
mod area;
#[cfg(feature = "docs")]
mod docs;

#[cfg(feature = "barcode")]
mod barcode;

mod sender;
mod site;
mod user;

#[options("/{_:.*}")]
pub(crate) async fn options() -> impl Responder {
    HttpResponse::Ok()
        .insert_header(("Access-Control-Allow-Origin", "*"))
        .insert_header(("Access-Control-Allow-Methods", "DELETE, POST, GET, OPTIONS"))
        .insert_header((
            "Access-Control-Allow-Headers",
            "Content-Type, Authorization, X-Requested-With",
        ))
        .finish()
}

pub(crate) fn router<T>(app: App<T>) -> App<T>
where
    T: ServiceFactory<ServiceRequest, Config = (), Error = Error, InitError = ()>,
{
    #[cfg(feature = "docs")]
    let app = app.service(
        scope("/api/docs")
            .service(docs::docs_setting)
            .service(docs::docs_raw)
            .service(docs::docs_read)
            .service(options),
    );
    #[cfg(feature = "barcode")]
    let app = {
        app.service(
            scope("/api/barcode")
                .service(barcode::barcode)
                .service(options),
        )
    };
   

    let mut user_scope=scope("/api/user")
    .service(user::user_list)
    .service(user::user_logs)
    .service(user::email)
    .service(user::email_confirm)
    .service(user::external)
    .service(user::set_info)
    .service(user::login)
    .service(user::user_data)
    .service(user::logout)
    .service(user::external_login_url)
    .service(user::external_login_callback)
    .service(user::external_state_check)
    .service(user::external_state_callback)
    .service(user::login_history)
    .service(user::mobile)
    .service(user::password_reset)
    .service(user::password)
    .service(user::res)
    .service(user::role)
    .service(user::access)
    .service(user::reg)
    .service(user::oauth)
    .service(options);
    

    #[cfg(feature = "area")]
    let app = {
        user_scope=user_scope.service(user::address);
        app .service(scope("/api/area").service(area::area_data).service(options))
    };
    
    app.service(user_scope)
    .service(
        scope("/api/setting")
            .service(site::oauth_config)
            .service(site::system_config)
            .service(options),
    )
    .service(
        scope("/api/site")
            .service(site::system_info)
            .service(options),
    )
    .service(scope("/api/app").service(app::app).service(options))
    .service(
        scope("/api/sender")
            .service(sender::sender_smser)
            .service(sender::sender_mailer)
            .service(sender::sender_tpl_body)
            .service(options),
    )
}
