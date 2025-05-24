use crate::common::handler::ReqQuery;
use crate::common::handler::{JsonQuery, ResponseJson, ResponseJsonResult};
use actix_files::NamedFile;
use actix_web::error::{ErrorInternalServerError, ErrorNotFound};
use actix_web::get;
use actix_web::post;
use actix_web::web;
use actix_web::CustomizeResponder;
use actix_web::Responder;
use actix_web::Result;
use lsys_web::handler::api::public::docs::{
    file_path, md_read, menu_data, MdReadParam, RawReadParam,
};
use tracing::debug;

#[get("/raw/{id}/{path}")]
pub async fn docs_raw(
    req_dao: ReqQuery,
    info: web::Path<(u32, String)>,
) -> Result<CustomizeResponder<NamedFile>> {
    let info = info.into_inner();
    let path = file_path(
        &RawReadParam {
            menu_id: info.0,
            url: info.1.to_owned(),
        },
        &req_dao,
    )
    .await
    .map_err(|err| {
        if matches!(
            err,
            lsys_web::lsys_docs::dao::GitDocError::Sqlx(lsys_web::sqlx::Error::RowNotFound)
        ) {
            return ErrorNotFound(req_dao.fluent_error_string(&err.into()));
        }
        ErrorInternalServerError(req_dao.fluent_error_string(&err.into()))
    })?;
    debug!("read raw file:{}", &path.file_path.to_string_lossy());
    let file = NamedFile::open_async(path.file_path).await?;
    let ftype = file.content_type().to_string();
    let mut res = file.customize().insert_header(("x-version", path.version));
    if !ftype.contains("charset") {
        res = res.insert_header(("Content-Type", format!("{};charset=utf-8", ftype)));
    }
    Ok(res)
}

#[post("/read/{type}")]
pub async fn docs_read(
    path: actix_web::web::Path<String>,
    req_dao: ReqQuery,
    json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    let res = match path.into_inner().as_str() {
        "menu" => menu_data(&req_dao).await,
        "md" => md_read(&json_param.param::<MdReadParam>()?, &req_dao).await,
        name => handler_not_found!(name),
    };
    Ok(res
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}
