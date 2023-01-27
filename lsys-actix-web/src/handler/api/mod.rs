use actix_service::ServiceFactory;
use actix_web::{dev::ServiceRequest, options, web::scope, App, Error, HttpResponse, Responder};

mod app;
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
    app.service(
        scope("/api/user")
            .service(user::address)
            .service(user::email)
            .service(user::email_confirm)
            .service(user::external)
            .service(user::set_info)
            .service(user::login)
            .service(user::user_data)
            .service(user::logout)
            .service(user::oauth_login)
            .service(user::oauth_callback)
            .service(user::oauth_qrcode_check)
            .service(user::oauth_qrcode_do_login)
            .service(user::login_history)
            .service(user::mobile)
            .service(user::password_reset)
            .service(user::password)
            .service(user::res)
            .service(user::role)
            .service(user::access)
            .service(user::reg)
            .service(user::oauth)
            .service(options),
    )
    .service(scope("/api/app").service(app::app).service(options))
}
