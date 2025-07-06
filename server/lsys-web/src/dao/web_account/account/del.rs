use crate::common::JsonResult;
use lsys_access::dao::SessionBody;
use lsys_core::RequestEnv;
use lsys_user::model::{AccountInfoModelRef, AccountModel, AccountStatus};

use super::WebUserAccount;
use lsys_core::model_option_set;

impl WebUserAccount {
    //删除当前登录用户
    pub async fn user_delete_from_session(
        &self,
        session: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        let account = self.user_dao.account_dao.session_account(session).await?;
        self.user_delete(&account, session, env_data).await?;
        self.user_dao.auth_dao.logout(session).await?;

        Ok(())
    }
    //删除用户
    pub async fn user_delete(
        &self,
        user: &AccountModel,
        session_body: &SessionBody,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<()> {
        if AccountStatus::Delete.eq(user.status) {
            return Ok(());
        }
        let mut tran = self.db.begin().await?;
        for email in self
            .user_dao
            .account_dao
            .account_email
            .find_by_account_id_vec(&user.id)
            .await?
        {
            let res = self
                .user_dao
                .account_dao
                .account_email
                .del_email(&email, session_body.user_id(), Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err.into());
            }
        }
        for mobile in self
            .user_dao
            .account_dao
            .account_mobile
            .find_by_account_id_vec(&user.id)
            .await?
        {
            let res = self
                .user_dao
                .account_dao
                .account_mobile
                .del_mobile(&mobile, session_body.user_id(), Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err.into());
            }
        }
        for external in self
            .user_dao
            .account_dao
            .account_external
            .find_by_account_id_vec(&user.id)
            .await?
        {
            let res = self
                .user_dao
                .account_dao
                .account_external
                .del_external(&external, session_body.user_id(), Some(&mut tran), env_data)
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err.into());
            }
        }
        let headimg = "".to_string();
        let info_ref = model_option_set!(AccountInfoModelRef,{
            headimg:headimg,
        });
        let res = self
            .user_dao
            .account_dao
            .account_info
            .set_info(
                user,
                &info_ref,
                session_body.user_id(),
                Some(&mut tran),
                env_data,
            )
            .await;
        if let Err(err) = res {
            tran.rollback().await?;
            return Err(err.into());
        }
        let res = self
            .user_dao
            .account_dao
            .account_name
            .remove_account_name(user, session_body.user_id(), Some(&mut tran), env_data)
            .await;
        if let Err(err) = res {
            tran.rollback().await?;
            return Err(err.into());
        }
        let res = self
            .user_dao
            .account_dao
            .account
            .del(
                user,
                Some("delete user"),
                session_body.user_id(),
                Some(&mut tran),
                env_data,
            )
            .await;
        if let Err(err) = res {
            tran.rollback().await?;
            return Err(err.into());
        }
        tran.commit().await?;
        Ok(())
    }
}
