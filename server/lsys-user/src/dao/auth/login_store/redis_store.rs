use crate::dao::auth::{
    LoginType, SessionData, SessionUserData, UserAuthData, UserAuthResult, UserAuthStore,
    UserAuthTokenData,
};
use crate::model::UserModel;
use async_trait::async_trait;
use lsys_core::{now_time, rand_str, RandType};
// use rand::seq::SliceRandom;

use redis::AsyncCommands;
use std::collections::HashMap;
use std::ops::Add;
use std::prelude::v1::Result::Err;
use std::result::Result::Ok;
use std::string::FromUtf8Error;

use std::time::SystemTime;

use tracing::{debug, trace, warn};

use super::super::{LoginData, UserAuthError};

pub struct UserAuthRedisStore {
    redis: deadpool_redis::Pool,
}
impl UserAuthRedisStore {
    pub fn new(redis: deadpool_redis::Pool) -> Self {
        Self { redis }
    }
}

const LOGIN_KEY_LEN: usize = 24;

fn login_key(size: usize) -> Result<String, FromUtf8Error> {
    Ok(rand_str(RandType::Upper, size))
    // const BASE_STR: &str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    // let mut rng = &mut rand::thread_rng();
    // String::from_utf8(
    //     BASE_STR
    //         .as_bytes()
    //         .choose_multiple(&mut rng, size)
    //         .cloned()
    //         .collect(),
    // )
}

#[async_trait]
/// 登录TOKEN存储到REDIS
impl UserAuthStore for UserAuthRedisStore {
    async fn set_data(
        &mut self,
        user_token_data: Option<UserAuthTokenData>,
        login_type: LoginType,
        login_data: LoginData,
        account: UserModel,
    ) -> UserAuthResult<UserAuthTokenData> {
        let key = format!("login::{}", account.id);
        trace!(login_id = %account.id,login_type=%login_type.type_name, "set login data");
        let now_time = now_time()?;
        let time_out = SystemTime::now()
            .add(std::time::Duration::from_secs(login_type.time_out as u64))
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_secs();
        let token = if let Some(dat) = user_token_data {
            dat
        } else {
            UserAuthTokenData::new(login_key(LOGIN_KEY_LEN)?, account.id, time_out)
        };
        let mut set_time_out: u64 = 0;
        let mut redis = self.redis.get().await?;
        let redis_data_opt: Option<HashMap<String, String>> = redis.hgetall(key.as_str()).await?;
        let redis_data = redis_data_opt.unwrap_or_default();
        debug!(login_id = %account.id,login_type=%login_type.type_name,"login count : {:?}",redis_data.len());
        for (login_key, login_item) in redis_data {
            if let Ok(item) = serde_json::from_str::<UserAuthData>(login_item.as_str()) {
                if now_time > item.user_data().time_out {
                    let res: Result<(), _> = redis.hdel(key.as_str(), login_key).await;
                    if let Err(err) = res {
                        warn!(login_id = %account.id,login_type=%login_type.type_name,"redis del error: {:?}",err);
                    }
                    continue;
                }
                if time_out < item.user_data().time_out {
                    set_time_out = item.user_data().time_out;
                }
            }
        }
        if set_time_out < time_out {
            set_time_out = time_out;
        }
        let user_auth_data = UserAuthData::new(
            SessionUserData {
                user_id: account.id,
                user_nickname: account.nickname,
                user_password_id: account.password_id,
                time_out,
            },
            login_type,
            login_data,
        );
        let val = serde_json::to_string(&user_auth_data)?;
        let _: () = redis.hset(key.as_str(), token.to_string(), val).await?;
        let _: () = redis.expire(key.as_str(), set_time_out as usize).await?;
        UserAuthResult::Ok(token)
    }
    async fn clear_data(&mut self, token: &UserAuthTokenData) -> UserAuthResult<()> {
        let key = format!("login::{}", token.user_id);
        debug!(login_id = %token.user_id,"login out");
        let mut redis = self.redis.get().await?;
        let _: () = redis.hdel(key, token.to_string()).await?;
        UserAuthResult::Ok(())
    }
    async fn get_data(&self, token: &UserAuthTokenData) -> UserAuthResult<UserAuthData> {
        let key = format!("login::{}", token.user_id);
        let mut redis = self.redis.get().await?;
        let user_data: Option<String> = redis.hget(key, token.to_string()).await?;
        match user_data {
            Some(data) => {
                let user_auth = serde_json::from_str::<UserAuthData>(data.as_str())?;
                Ok(user_auth)
            }
            None => Err(UserAuthError::NotLogin("token not find".to_string())),
        }
    }
    async fn exist_data(&self, token: &UserAuthTokenData) -> UserAuthResult<bool> {
        let key = format!("login::{}", token.user_id);
        let mut redis = self.redis.get().await?;
        let is_login = redis.hexists(key, token.to_string()).await.unwrap_or(false);
        return Ok(is_login);
    }
}
