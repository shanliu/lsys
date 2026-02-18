use crate::common::JsonData;
use crate::dao::{WebError, WebResult};

use super::WebUserAccount;
use lsys_access::dao::SessionBody;
use lsys_core::{fluent_message, IntoFluentMessage, RequestEnv};

use lsys_user::dao::{AccountError, AccountInfoParam};
use lsys_user::model::AccountPasswordModel;
use tracing::warn;

impl WebUserAccount {
    pub async fn user_info_set_username(
        &self,
        name: &str,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> WebResult<()> {
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
        if let Err(err) = self
            .access_dao
            .user
            .sync_user(0, account.id, None, Some(name))
            .await
        {
            warn!(
                "sync user account to access fail:{}",
                err.to_fluent_message().default_format()
            );
        };

        Ok(())
    }
}

impl WebUserAccount {
    pub async fn user_info_check_username(&self, name: &str) -> WebResult<()> {
        let user_res = self
            .user_dao
            .account_dao
            .account_name
            .find_by_name(name)
            .await;
        match user_res {
            Err(AccountError::Sqlx(sqlx::Error::RowNotFound)) => Ok(()),
            Err(err) => Err(err.into()),
            Ok(user) => Err(WebError::JsonResponse(
                Box::new(JsonData::default().set_sub_code("username_exists")),
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
    ) -> WebResult<()> {
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
        let headimg = param.headimg.map(|e| e.to_string());
        let birthday = param.birthday.map(|e| e.to_string());
        let info_param = AccountInfoParam {
            gender: param.gender,
            headimg: headimg.as_deref(),
            birthday: birthday.as_deref(),
            ..Default::default()
        };
        let res = self
            .user_dao
            .account_dao
            .account_info
            .set_info(
                &account,
                &info_param,
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

        if let Some(nikename) = param.nikename {
            //此过程必须,通过过去好查数据
            if let Err(err) = self
                .access_dao
                .user
                .sync_user(0, account.id, Some(nikename), None)
                .await
            {
                warn!(
                    "sync user nikename to access fail:{}",
                    err.to_fluent_message().default_format()
                );
            };
        }

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
    ) -> WebResult<(AccountPasswordModel, bool, u64)> {
        //密码记录,是否超时,密码超时时长
        let account = self
            .user_dao
            .account_dao
            .session_account(session_body)
            .await?;
        if account.password_id == 0 {
            return Err(WebError::Message(fluent_message!("password-not-set")));
        }
        let user = self
            .user_dao
            .account_dao
            .account_password
            .find_by_id(&account.password_id)
            .await?;
        let (is_passwrod_timeout, password_timeout_value) = self
            .user_dao
            .account_dao
            .account_password
            .password_timeout(account.id)
            .await
            .unwrap_or((false, 0));
        Ok((user, is_passwrod_timeout, password_timeout_value))
    }
}
