use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery, ReqQuery};
use actix_web::{post, web};
use lsys_web::dao::WebDao;
use lsys_web::handler::rest::rbac::{
    ResAddParam, ResDelOpParam, ResDelParam, ResEditParam, ResParam, ResTypeAddOpParam,
    ResTypeListParam, ResTypeOpListParam, res_add, res_data, res_del, res_edit, res_type_data,
    res_type_op_add, res_type_op_data, res_type_op_del,
};

#[post("/res")]
pub async fn res(rest: RestQuery, req_dao: ReqQuery, web_dao: web::Data<WebDao>) -> ResponseJsonResult<ResponseJson> {
    let data = match rest.rfc.method.as_deref().unwrap_or_default() {
        "add" => res_add(&rest.param::<ResAddParam>()?, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await,
        "edit" => {
            res_edit(
                &rest.param::<ResEditParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "delete" => res_del(&rest.param::<ResDelParam>()?, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await,
        "list" => res_data(&rest.param::<ResParam>()?, &rest.get_app().await?, &req_dao, web_dao.as_ref()).await,
        "type_data" => {
            res_type_data(
                &rest.param::<ResTypeListParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "type_op_add" => {
            res_type_op_add(
                &rest.param::<ResTypeAddOpParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "type_op_del" => {
            res_type_op_del(
                &rest.param::<ResDelOpParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        "type_op_data" => {
            res_type_op_data(
                &rest.param::<ResTypeOpListParam>()?,
                &rest.get_app().await?,
                &req_dao,
                web_dao.as_ref(),
            )
            .await
        }
        name => handler_not_found!(name),
    };
    Ok(data
        .map_err(|e| req_dao.fluent_error_json_response(&e))?
        .into())
}
