//使用rest接口登陆后，根据生成的登陆code，完成在系统后台登陆的实现
use lsys_access::dao::{AccessAuthLoginData, AccessDao, AccessLoginData, SessionBody};
use lsys_core::{
    valid_key, AppCore, ValidNumber, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen,
};

use std::sync::Arc;

use super::{AccountResult, UserAuthToken};

pub struct AuthCode {
    access: Arc<AccessDao>,
    app_core: Arc<AppCore>,
}
pub const CODE_LOGIN_TYPE: &str = "code";

impl AuthCode {
    pub fn new(access: Arc<AccessDao>, app_core: Arc<AppCore>) -> Self {
        Self { access, app_core }
    }
    pub async fn code_login(
        &self,
        app_id: u64,
        token_code: &str,
        user_data: &str,
        user_nickname: &str,
        login_data: &AccessLoginData<'_>,
    ) -> AccountResult<SessionBody> {
        ValidParam::default()
            .add(
                valid_key!("app_id"),
                &app_id,
                &ValidParamCheck::default().add_rule(ValidNumber::id()),
            )
            .add(
                valid_key!("token_code"),
                &token_code,
                &ValidParamCheck::default()
                    .add_rule(ValidStrlen::range(16, 128))
                    .add_rule(ValidPattern::Ident),
            )
            .check()?;
        let app_jwt_key = self
            .app_core
            .config
            .find(None)
            .get_string("app_jwt_key")
            .unwrap_or_default();
        let token_data = format!(
            "{:x}",
            md5::compute(format!("{}-{}-{}", app_id, token_code, app_jwt_key))
        );
        let res = self
            .access
            .auth
            .do_login(&AccessAuthLoginData {
                app_id,
                oauth_app_id: 0,
                user_data,
                user_nickname,
                token_data: Some(&token_data),
                login_type: CODE_LOGIN_TYPE,
                login_data: Some(login_data),
            })
            .await?;
        Ok(res)
    }
    pub async fn code_logout(&self, app_id: u64, token_data: &str) -> AccountResult<()> {
        let session = self.access.auth.login_data(app_id, 0, token_data).await?;
        Ok(self.access.auth.do_logout(&session).await?)
    }
    pub async fn login_data(&self, app_id: u64, token_data: &str) -> AccountResult<SessionBody> {
        Ok(self
            .access
            .auth
            .cache()
            .login_data(app_id, 0, token_data)
            .await?)
    }
    pub fn to_token(session: &SessionBody) -> UserAuthToken {
        UserAuthToken::new(
            session.session().user_app_id,
            session.token_data(),
            session.user_id(),
            session.session().expire_time,
        )
    }
}
