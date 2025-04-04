//使用rest接口登陆后，根据生成的登陆code，完成在系统后台登陆的实现
use lsys_access::dao::{AccessAuthLoginData, AccessDao, AccessLoginData, SessionBody};
use lsys_core::AppCore;

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
        user_name: &str,
        login_data: &AccessLoginData<'_>,
    ) -> AccountResult<SessionBody> {
        let app_jwt_key = self
            .app_core
            .config
            .find(None)
            .get_string("app_jwt_key")
            .unwrap_or_default();
        let login_code = format!(
            "{:x}",
            md5::compute(format!("{}-{}-{}", app_id, token_code, app_jwt_key))
        );
        Ok(self
            .access
            .auth
            .do_login(&AccessAuthLoginData {
                app_id,
                oauth_app_id: 0,
                user_data,
                user_name,
                token_data: Some(&login_code),
                login_type: CODE_LOGIN_TYPE,
                login_data: Some(login_data),
            })
            .await?)
    }
    pub async fn code_logout(&self, app_id: u64, login_code: &str) -> AccountResult<()> {
        let session = self.access.auth.login_data(app_id, 0, login_code).await?;
        Ok(self.access.auth.do_logout(&session).await?)
    }
    pub async fn login_data(&self, app_id: u64, login_code: &str) -> AccountResult<SessionBody> {
        Ok(self
            .access
            .auth
            .cache()
            .login_data(app_id, 0, login_code)
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
    //图片验证码 待定....
    // fn valid_code(&self) -> lsys_core::ValidCode {
    //     lsys_core::ValidCode::new(self.redis.clone(), VAILD_CODE.to_string(), true)
    // }
    // /// 获取验证码
    // pub async fn valid_code_set(&self, login_code: &str) -> AccountResult<(String, usize)> {
    //     let mut valid_code_data =
    //         lsys_core::ValidCodeDataRandom::new(lsys_core::ValidCodeTime::time(60));
    //     let res = self
    //         .valid_code()
    //         .delay_code(login_code, &mut valid_code_data)
    //         .await;
    //     match res {
    //         Ok(out) => Ok(out),
    //         Err(lsys_core::ValidCodeError::DelayTimeout(_)) => {
    //             let out = self
    //                 .valid_code()
    //                 .set_code(login_code, &mut valid_code_data)
    //                 .await?;
    //             Ok(out)
    //         }
    //         Err(err) => Err(err.into()),
    //     }
    // }
    // // /// 检测验证码
    // pub async fn valid_code_check(&self, login_code: &str, valid_code: &str) -> AccountResult<()> {
    //     self.valid_code().check_code(login_code, valid_code).await?;
    //     Ok(())
    // }
    // pub async fn valid_code_clear(&self, login_code: &str) -> AccountResult<()> {
    //     let mut builder = lsys_core::ValidCodeDataRandom::new(lsys_core::ValidCodeTime::time(60));
    //     self.valid_code()
    //         .clear_code(login_code, &mut builder)
    //         .await?;
    //     Ok(())
    // }
}
