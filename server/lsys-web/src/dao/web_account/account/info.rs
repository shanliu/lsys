use crate::common::JsonData;
use crate::common::{JsonError, JsonResult};

use super::WebUserAccount;
use lsys_access::dao::SessionBody;
use lsys_core::model_option_set;
use lsys_core::{fluent_message, RequestEnv};

use lsys_user::model::AccountPasswordModel;
use lsys_user::{dao::AccountError, model::AccountInfoModelRef};

impl WebUserAccount {
    pub async fn user_info_set_username(
        &self,
        name: &str,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;
        self.user_dao
            .account_dao
            .account_name
            .change_account_name(&account, name, session_body.user_id(), None, env_data)
            .await?;

        Ok(())
    }
}

impl WebUserAccount {
    pub async fn user_info_check_username(&self, name: &str) -> JsonResult<()> {
        let user_res = self
            .user_dao
            .account_dao
            .account_name
            .find_by_name(name)
            .await;
        match user_res {
            Err(AccountError::Sqlx(sqlx::Error::RowNotFound)) => Ok(()),
            Err(err) => Err(err.into()),
            Ok(user) => Err(JsonError::JsonResponse(
                JsonData::default().set_sub_code("username_exists"),
                fluent_message!("username-is-exists",{
                    "id":user.id
                }),
            )),
        }
    }
}

pub struct InfoSetUserInfoData<'t> {
    pub nikename: Option<&'t str>,
    pub gender: Option<i32>,
    pub headimg: Option<&'t str>,
    pub birthday: Option<&'t str>,
}
impl WebUserAccount {
    pub async fn user_info_set_data(
        &self,
        param: &InfoSetUserInfoData<'_>,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;
        let mut db = self.db.begin().await?;
        if let Some(nikename) = param.nikename {
            let res = self
                .user_dao
                .account_dao
                .account
                .set_nikename(
                    &account,
                    nikename,
                    session_body.user_id(),
                    Some(&mut db),
                    env_data,
                )
                .await;
            if let Err(err) = res {
                db.rollback().await?;
                return Err(err.into());
            }
        }
        let mut info = model_option_set!(AccountInfoModelRef, {});
        info.gender = param.gender.as_ref();
        let headimg = param.headimg.map(|e| e.to_string());
        info.headimg = headimg.as_ref();
        let birthday = param.birthday.map(|e| e.to_string());
        info.birthday = birthday.as_ref();
        let res = self
            .user_dao
            .account_dao
            .account_info
            .set_info(
                &account,
                &info,
                session_body.user_id(),
                Some(&mut db),
                env_data,
            )
            .await;
        if let Err(err) = res {
            db.rollback().await?;
            return Err(err.into());
        }
        db.commit().await?;
        // let token = self
        //     .user_dao
        //     .auth_dao
        //     .reload(user_session.read().await.get_session_token())
        //     .await?;
        Ok(())
    }
}
impl WebUserAccount {
    pub async fn password_last_modify(
        &self,
        session_body: &SessionBody,
    ) -> JsonResult<(AccountPasswordModel, bool)> {
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;
        if account.password_id == 0 {
            return Err(JsonError::Message(fluent_message!("password-not-set")));
        }
        let user = self
            .user_dao
            .account_dao
            .account_password
            .find_by_id(&account.password_id)
            .await?;
        let passwrod_timeout = self
            .user_dao
            .account_dao
            .account_password
            .password_timeout(account.id)
            .await
            .unwrap_or(false);
        Ok((user, passwrod_timeout))
    }
}
