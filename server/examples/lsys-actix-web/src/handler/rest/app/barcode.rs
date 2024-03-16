use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;

#[post("barcode")]
pub(crate) async fn mail(mut _rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    todo!()
    // Ok(match rest.rfc.method.as_deref() {
    //     Some("parse") => {
    //         todo!()
    //     }
    //     Some("create") => {
    //         todo!()
    //     }
    //     var => handler_not_found!(var.unwrap_or_default()),
    // }?
    // .into())
}
