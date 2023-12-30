use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use actix_web::web::Data;
use lsys_web::{
    dao::WebDao,
    handler::app::{subapp_view, SubAppViewParam},
};

// 请求   -> 模块
//       -> 系统分配appid
//       -> 系统[访问用户+查询指定appid组成的关系key,检查权限[资源id:global-app-access-{appid}]]
//       -> 返回查询appid密钥
//       -> 检查签名
//       -> 授权查询...
#[post("app")]
pub(crate) async fn app(
    mut rest: RestQuery,
    app_dao: Data<WebDao>,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("view") => {
            let param = rest.param::<SubAppViewParam>()?;
            subapp_view(
                &app_dao,
                &rest.rfc.to_app_model(&app_dao.app.app_dao.app).await?,
                param,
            )
            .await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
