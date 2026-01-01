use crate::common::handler::WebHandError;

use actix_web::web::Data;
use actix_web::Result;
use actix_web::{get, HttpResponse};
use lsys_web::dao::WebDao;

#[get("/dome")]
pub(crate) async fn dome(web_dao: Data<WebDao>) -> Result<HttpResponse, WebHandError> {
    let mut ctx = lsys_web::tera::Context::new();
    ctx.insert("name", &"lsys".to_owned());
    ctx.insert("text", &"Welcome!".to_owned());
    let body = web_dao.tera.render("index.html", &ctx)?;
    Ok(HttpResponse::Ok().content_type("text/html").body(body))
}
