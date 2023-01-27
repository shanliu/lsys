use std::sync::Arc;

use crate::dao::account::UserAccountResult;
use crate::dao::auth::UserPasswordHash;

use crate::model::{UserModel, UserModelRef, UserPasswordModel, UserPasswordModelRef};
use lsys_core::{get_message, now_time, FluentMessage};
use redis::aio::ConnectionManager;
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::{model_option_set, Insert,  Select, SqlQuote, Update};
use tokio::sync::Mutex;
use tracing::warn;

use super::UserAccountError;

pub struct UserPassword {
    db: Pool<MySql>,
    fluent: Arc<FluentMessage>,
    redis: Arc<Mutex<ConnectionManager>>,
    user_passwrd_hash: Arc<UserPasswordHash>,
}

impl UserPassword {
    pub fn new(
        db: Pool<MySql>,
        fluent: Arc<FluentMessage>,
        redis: Arc<Mutex<ConnectionManager>>,
        user_passwrd_hash: Arc<UserPasswordHash>,
    ) -> Self {
        Self {
            db,
            fluent,
            redis,
            user_passwrd_hash,
        }
    }
    impl_account_valid_code_method!("passwrod",{
        user_id: &u64,
        from_type: &String,
    },{
        format!("{}-{}",user_id,from_type)
    },5*60);
    /// 校验验证码并设置新密码
    pub async fn set_passwrod_from_code<'t>(
        &self,
        user: &UserModel,
        new_password: String,
        from_type: &String,
        code: &String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        self.valid_code_check(code, &user.id, from_type).await?;
        let res = self.set_passwrod(user, new_password, transaction).await;
        if res.is_ok() {
            if let Err(err) = self.valid_code_clear(&user.id, from_type).await {
                warn!("email {} valid clear fail:{}", &user.id, err);
            }
        }
        res
    }
    /// 设置新密码
    pub async fn set_passwrod<'t>(
        &self,
        user: &UserModel,
        new_password: String,
        transaction: Option<&mut Transaction<'t, sqlx::MySql>>,
    ) -> UserAccountResult<u64> {
        let new_password = new_password.trim().to_string();
        if new_password.len() < 6 || new_password.len() > 32 {
            return Err(UserAccountError::System(get_message!(
                &self.fluent,
                "user-passwrod-wrong",
                "password length need 6-32 char"
            )));
        }
        let nh_passwrod = self.user_passwrd_hash.hash_password(&new_password).await;
        let db = &self.db;
        let time = now_time()?;
        let mut ta;
        if user.password_id > 0 {
            let user_pass_res = Select::type_new::<UserPasswordModel>()
                .fetch_one_by_where_call::<UserPasswordModel, _, _>(
                    "user_id=? and id=?".to_string(),
                    |mut res, _| {
                        res = res.bind(user.id);
                        res = res.bind(user.password_id);
                        res
                    },
                    db,
                )
                .await;
            match user_pass_res {
                Err(sqlx::Error::RowNotFound) => {
                    ta = match transaction {
                        Some(pb) => pb.begin().await?,
                        None => db.begin().await?,
                    };
                }
                Ok(user_pass) => {
                    ta = match transaction {
                        Some(pb) => pb.begin().await?,
                        None => db.begin().await?,
                    };
                    let change =
                        sqlx_model::model_option_set!(UserPasswordModelRef, { change_time: time });
                    //ta.execute(query)
                    Update::<sqlx::MySql, UserPasswordModel, _>::new(change)
                        .execute_by_pk(&user_pass, &mut ta)
                        .await?;
                }
                Err(err) => {
                    return Err(err.into());
                }
            }
        } else {
            ta = match transaction {
                Some(pb) => pb.begin().await?,
                None => db.begin().await?,
            };
        }

        let new_data = model_option_set!(UserPasswordModelRef,{
            user_id:user.id,
            password:nh_passwrod,
            change_time: time,
            add_time: time,
        });
        let res = Insert::<sqlx::MySql, UserPasswordModel, _>::new(new_data)
            .execute(&mut ta)
            .await;
        match res {
            Err(e) => {
                ta.rollback().await?;
                Err(e.into())
            }
            Ok(data) => {
                let pid = data.last_insert_id();
                let change = sqlx_model::model_option_set!(UserModelRef,{
                    password_id:pid,
                });
                let u_res = Update::<sqlx::MySql, UserModel, _>::new(change)
                    .execute_by_pk(user, &mut ta)
                    .await;
                match u_res {
                    Err(e) => {
                        ta.rollback().await?;
                        Err(e.into())
                    }
                    Ok(_) => {
                        ta.commit().await?;
                        Ok(pid)
                    }
                }
            }
        }
    }
    lsys_core::impl_dao_fetch_one_by_one!(
        db,
        find_by_id,
        u64,
        UserPasswordModel,
        UserAccountResult<UserPasswordModel>,
        id,
        "id = {id} "
    );
    /// 检测密码是否正确
    pub async fn check_password(
        &self,
        user: &UserModel,
        check_password: &String,
    ) -> UserAccountResult<bool> {
        let user_password =self.find_by_id(&user.password_id).await?;
        Ok(self.user_passwrd_hash.hash_password(check_password).await == user_password.password)
    }
}
