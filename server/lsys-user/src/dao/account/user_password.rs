use std::sync::Arc;

use crate::dao::account::UserAccountResult;
use crate::dao::auth::UserPasswordHash;

use crate::model::{UserModel, UserModelRef, UserPasswordModel, UserPasswordModelRef};
use lsys_core::{fluent_message, now_time, IntoFluentMessage};

use lsys_setting::dao::{
    NotFoundResult, SettingDecode, SettingEncode, SettingJson, SettingKey, SettingResult,
    SingleSetting,
};
use serde::{Deserialize, Serialize};
use sqlx::{Acquire, MySql, Pool, Transaction};
use sqlx_model::SqlQuote;
use sqlx_model::{model_option_set, sql_format, Insert, Select, Update};
use tracing::warn;

use super::UserAccountError;

#[derive(Deserialize, Serialize, Clone, Default)]
pub struct UserPasswordConfig {
    pub timeout: u64,
    pub disable_old_password: bool,
}

impl SettingKey for UserPasswordConfig {
    fn key<'t>() -> &'t str {
        "user-password"
    }
}
impl SettingDecode for UserPasswordConfig {
    fn decode(data: &str) -> SettingResult<Self> {
        SettingJson::decode(data)
    }
}
impl SettingEncode for UserPasswordConfig {
    fn encode(&self) -> String {
        SettingJson::encode(self)
    }
}
impl SettingJson<'_> for UserPasswordConfig {}

pub struct UserPassword {
    db: Pool<MySql>,
    // fluent: Arc<FluentBuild>,
    redis: deadpool_redis::Pool,
    user_passwrd_hash: Arc<UserPasswordHash>,
    setting: Arc<SingleSetting>,
}

impl UserPassword {
    pub fn new(
        db: Pool<MySql>,
        setting: Arc<SingleSetting>,
        //fluent: Arc<FluentBuild>,
        redis: deadpool_redis::Pool,
        user_passwrd_hash: Arc<UserPasswordHash>,
    ) -> Self {
        Self {
            db,
            // fluent,
            redis,
            user_passwrd_hash,
            setting,
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
                warn!(
                    "email {} valid clear fail:{}",
                    &user.id,
                    err.to_fluent_message().default_format()
                );
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
            return Err(UserAccountError::System(
                fluent_message!("user-passwrod-wrong",
                    {
                        "len":new_password.len(),
                        "min":6,
                        "max":32
                    }
                ),
            )); //"password length need 6-32 char"
        }

        let db = &self.db;
        let time = now_time()?;
        let mut ta;
        if user.password_id > 0 {
            let user_pass_res = Select::type_new::<UserPasswordModel>()
                .fetch_one_by_where::<UserPasswordModel, _>(
                    &sqlx_model::WhereOption::Where(sql_format!(
                        "user_id={} and id={}",
                        user.id,
                        user.password_id,
                    )),
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
                        sqlx_model::model_option_set!(UserPasswordModelRef, { disable_time: time });
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
        let nh_passwrod = self.user_passwrd_hash.hash_password(&new_password).await;

        let config = self
            .setting
            .load::<UserPasswordConfig>(&None)
            .await
            .notfound_default()?;

        if config.disable_old_password {
            let old_pass_res = Select::type_new::<UserPasswordModel>()
                .fetch_one_by_where::<UserPasswordModel, _>(
                    &sqlx_model::WhereOption::Where(sql_format!(
                        "user_id={} and password={}",
                        user.id,
                        nh_passwrod
                    )),
                    db,
                )
                .await;
            if old_pass_res.is_ok() {
                return Err(UserAccountError::System(fluent_message!(
                    "user-old-passwrod"
                ))); //                    "can't old password"
            }
        }

        let new_data = model_option_set!(UserPasswordModelRef,{
            user_id:user.id,
            password:nh_passwrod,
            disable_time: 0,
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
                    change_time:time,
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
        let user_password = match self.find_by_id(&user.password_id).await {
            Ok(up) => up,
            Err(err) => match err {
                UserAccountError::Sqlx(sqlx::Error::RowNotFound) => {
                    return Err(UserAccountError::System(fluent_message!(
                        "user-passwrod-delete"
                    ))); //"can't password,may be is delete"
                }
                _ => return Err(err),
            },
        };
        Ok(self.user_passwrd_hash.hash_password(check_password).await == user_password.password)
    }
    /// 检测指定ID密码是否超时
    pub async fn password_timeout(&self, password_id: &u64) -> UserAccountResult<bool> {
        if let Ok(set) = self
            .setting
            .load::<UserPasswordConfig>(&None)
            .await
            .notfound_default()
        {
            if set.timeout == 0 {
                return Ok(false);
            }
            if let Ok(password) = self.find_by_id(password_id).await {
                if password.add_time + set.timeout < now_time().unwrap_or_default() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
