use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_web::post;
use lsys_web::handler::app::{subapp_view, SubAppViewParam};

// 请求   -> 模块
//       -> 系统分配appid
//       -> 系统[访问用户+查询指定appid组成的关系key,检查权限[资源id:global-app-access-{appid}]]
//       -> 返回查询appid密钥
//       -> 检查签名
//       -> 授权查询...
#[post("app")]
pub(crate) async fn app(mut rest: RestQuery) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("view") => {
            let param = rest.param::<SubAppViewParam>()?;
            subapp_view(&rest, &rest.to_app_model().await?, param).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
