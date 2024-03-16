use crate::common::handler::{ResponseJson, ResponseJsonResult};
use actix_web::post;

use actix_web::get;

#[get("/{id}/{format}/{data}")]
async fn qrcode_show(// web_dao: Data<WebDao>,
    // info: web::Path<(u64, String, String)>,
) -> ResponseJsonResult<ResponseJson> {
    todo!()
}

#[post("/list")]
pub async fn qrcode_list(// path: actix_web::web::Path<String>,
    // auth_dao: UserAuthQuery,
    // json_param: JsonQuery,
    // jwt: JwtQuery,
) -> ResponseJsonResult<ResponseJson> {
    todo!()
}

#[post("/create")]
pub async fn create_code(// path: actix_web::web::Path<String>,
    // req_dao: ReqQuery,
    // json_param: JsonQuery,
) -> ResponseJsonResult<ResponseJson> {
    todo!()
}
