use lsys_core::RequestEnv;
use lsys_user::{
    dao::account::UserAccountResult,
    model::{UserEmailStatus, UserInfoModelRef, UserMobileStatus, UserModel, UserStatus},
};

use super::WebUser;

pub struct UserRegData<'a> {
    pub nikename: String,
    pub passwrod: Option<String>,
    pub name: Option<String>,
    pub email: Option<(String, UserEmailStatus)>,
    pub mobile: Option<(String, String, UserMobileStatus)>,
    pub external: Option<(String, String, String, String)>,
    pub info: Option<UserInfoModelRef<'a>>,
}

impl WebUser {
    // 注册用户
    pub async fn reg_user<'a>(
        &self,
        reg_data: UserRegData<'a>,
        env_data: Option<&RequestEnv>,
    ) -> UserAccountResult<UserModel> {
        let mut tran = self.db.begin().await?;
        let user = self
            .user_dao
            .user_account
            .user
            .add_user(
                reg_data.nikename,
                UserStatus::Enable,
                Some(&mut tran),
                env_data,
            )
            .await?;
        if let Some(pw) = reg_data.passwrod {
            let res = self
                .user_dao
                .user_account
                .user_password
                .set_passwrod(&user, pw, Some(&mut tran))
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some((un, st)) = reg_data.email {
            let res = self
                .user_dao
                .user_account
                .user_email
                .add_email(&user, un, st, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some(un) = reg_data.name {
            let res = self
                .user_dao
                .user_account
                .user_name
                .change_username(&user, un, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some((area, mob, st)) = reg_data.mobile {
            let res = self
                .user_dao
                .user_account
                .user_mobile
                .add_mobile(&user, area, mob, st, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some((config_name, external_type, external_id, external_name)) = reg_data.external {
            let res = self
                .user_dao
                .user_account
                .user_external
                .add_external(
                    &user,
                    config_name,
                    external_type,
                    external_id,
                    external_name,
                    Some(&mut tran),
                    env_data,
                )
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        if let Some(info_ref) = reg_data.info {
            let res = self
                .user_dao
                .user_account
                .user_info
                .set_info(&user, info_ref, Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        tran.commit().await?;
        Ok(user)
    }
}
