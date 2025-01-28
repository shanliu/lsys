use lsys_app::model::AppModel;
use lsys_rbac::dao::{AccessCheckEnv, AccessCheckRes, RbacAccess, RbacResult};
use lsys_web::{
    common::{JsonData, JsonResult, RequestDao},
    dao::{CheckRelationData, RbacCheckAccess},
};
use serde::Deserialize;
use serde_json::json;

//这里定义访问权限验证
pub struct DomeAccess {
    pub app: AppModel,
}

#[async_trait::async_trait]
impl RbacCheckAccess for DomeAccess {
    async fn check(
        &self,
        access: &RbacAccess,
        check_env: &AccessCheckEnv<'_>,
        relation: &CheckRelationData,
    ) -> RbacResult<()> {
        access
            .check(
                check_env,                   //资源访问用户
                &relation.to_session_role(), //资源关系
                &[AccessCheckRes::system_empty_data(
                    &format!("app-{}", self.app.id), //资源KEY
                    vec!["global-dome-auth"],        //必须验证权限
                )],
            )
            .await
    }
}

#[derive(Debug, Deserialize)]
pub struct DemoParam {
    pub text: String,
}
pub async fn demo_handler(
    param: &DemoParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    //验证权限
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env(),
            &DomeAccess {
                app: app.to_owned(),
            },
            None,
        )
        .await?;
    //业务逻辑。。。
    Ok(JsonData::data(json!({ "text":param.text })))
}
