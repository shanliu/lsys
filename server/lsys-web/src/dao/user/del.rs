use lsys_user::{
    dao::account::UserAccountResult,
    model::{UserInfoModelRef, UserStatus},
};

use sqlx_model::model_option_set;

use super::WebUser;

impl WebUser {
    //删除用户
    pub async fn del_user(&self, id: &u64) -> UserAccountResult<()> {
        let user = self.user_dao.user_account.user.find_by_id(id).await?;
        if UserStatus::Delete.eq(user.status) {
            return Ok(());
        }
        let mut tran = self.db.begin().await?;
        for email in self
            .user_dao
            .user_account
            .user_email
            .find_by_user_id_vec(&user.id)
            .await?
        {
            let res = self
                .user_dao
                .user_account
                .user_email
                .del_email(&email, Some(&mut tran))
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        for mobile in self
            .user_dao
            .user_account
            .user_mobile
            .find_by_user_id_vec(&user.id)
            .await?
        {
            let res = self
                .user_dao
                .user_account
                .user_mobile
                .del_mobile(&mobile, Some(&mut tran))
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        for external in self
            .user_dao
            .user_account
            .user_external
            .find_by_user_id_vec(&user.id)
            .await?
        {
            let res = self
                .user_dao
                .user_account
                .user_external
                .del_external(&external, Some(&mut tran))
                .await;
            if let Err(err) = res {
                tran.rollback().await?;
                return Err(err);
            }
        }
        let headimg = "".to_string();
        let info_ref = model_option_set!(UserInfoModelRef,{
            headimg:headimg,
        });
        let res = self
            .user_dao
            .user_account
            .user_info
            .set_info(&user, info_ref, Some(&mut tran))
            .await;
        if let Err(err) = res {
            tran.rollback().await?;
            return Err(err);
        }
        let res = self
            .user_dao
            .user_account
            .user
            .del_user(&user, Some(String::from("delete user")), Some(&mut tran))
            .await;
        if let Err(err) = res {
            tran.rollback().await?;
            return Err(err);
        }
        tran.commit().await?;
        Ok(())
    }
}
