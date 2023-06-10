use std::{
    fmt::{Display, Formatter},
    string::FromUtf8Error,
};

use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Connection, PoolError};
use rand::prelude::SliceRandom;
use redis::RedisError;
const CODE_SAVE_KEY: &str = "valid-save";

pub struct ValidCode {
    prefix: String,
    ignore_case: bool,
    redis: deadpool_redis::Pool,
}
#[derive(Debug)]
//不匹配错误
pub struct ValidCodeCheckError {
    pub message: String,
    pub prefix: String,
}
#[derive(Debug)]
pub enum ValidCodeError {
    Utf8Err(String),
    Create(String),
    Redis(String),
    Tag(String),
    DelayTimeout(ValidCodeCheckError),
    NotMatch(ValidCodeCheckError),
}
impl Display for ValidCodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl From<FromUtf8Error> for ValidCodeError {
    fn from(err: FromUtf8Error) -> Self {
        ValidCodeError::Utf8Err(err.to_string())
    }
}
impl From<RedisError> for ValidCodeError {
    fn from(err: RedisError) -> Self {
        ValidCodeError::Redis(err.to_string())
    }
}
impl From<PoolError> for ValidCodeError {
    fn from(err: PoolError) -> Self {
        ValidCodeError::Redis(err.to_string())
    }
}

#[async_trait]
pub trait ValidCodeData {
    async fn get_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        code: &'t Option<String>,
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<String>;
    async fn clear_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<()>;
    fn save_time(&self) -> usize;
}

pub struct ValidCodeTime {
    pub save_time: isize,
    pub duration_time: isize,
}
impl ValidCodeTime {
    pub fn time(time: isize) -> Self {
        Self {
            save_time: time,
            duration_time: time,
        }
    }
}

const CODE_CHANGE_KEY: &str = "valid-change";

pub struct ValidCodeDataRandom {
    valid_code_time: ValidCodeTime,
}
impl Default for ValidCodeDataRandom {
    fn default() -> Self {
        Self::new(ValidCodeTime {
            save_time: 120,
            duration_time: 60,
        })
    }
}
impl ValidCodeDataRandom {
    pub fn new(valid_code_time: ValidCodeTime) -> Self {
        Self { valid_code_time }
    }
    fn create_code(&self) -> ValidCodeResult<String> {
        const BASE_STR: &str = "0123456789";
        let mut rng = &mut rand::thread_rng();
        Ok(String::from_utf8(
            BASE_STR
                .as_bytes()
                .choose_multiple(&mut rng, 6)
                .cloned()
                .collect(),
        )?)
    }
}
#[async_trait]
impl ValidCodeData for ValidCodeDataRandom {
    async fn get_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        code: &'t Option<String>,
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<String> {
        let duration_time = self.valid_code_time.duration_time;
        let change_key = CODE_CHANGE_KEY.to_owned() + prefix + tag;
        let change_code: Option<String> = redis.get(change_key.as_str()).await?;
        let old_code = code.to_owned().unwrap_or_default();
        let out_code =
            if code.is_none() || old_code.is_empty() || change_code.unwrap_or_default() != old_code
            {
                self.create_code()?
            } else {
                old_code
            };

        if self.save_time() > 0 && duration_time > 0 {
            redis.set(change_key.as_str(), out_code.clone()).await?;
            redis
                .expire(change_key.as_str(), duration_time as usize)
                .await?;
        }
        Ok(out_code)
    }

    async fn clear_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<()> {
        let change_key = CODE_CHANGE_KEY.to_owned() + prefix + tag;
        redis.del(change_key).await?;
        Ok(())
    }

    fn save_time(&self) -> usize {
        if self.valid_code_time.save_time < self.valid_code_time.duration_time {
            self.valid_code_time.duration_time as usize
        } else {
            self.valid_code_time.save_time as usize
        }
    }
}

pub type ValidCodeResult<T> = Result<T, ValidCodeError>;

impl ValidCode {
    pub fn new(redis: deadpool_redis::Pool, prefix: String, ignore_case: bool) -> ValidCode {
        ValidCode {
            prefix,
            redis,
            ignore_case,
        }
    }
    pub async fn get_code(&self, tag: &String) -> ValidCodeResult<(String, usize)> {
        if tag.len() > 255 {
            return Err(ValidCodeError::Tag(format!("tag over length:{}", tag)));
        }
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let code: Option<String> = redis.get(save_key.as_str()).await?;
        let ttl = redis.ttl(save_key.as_str()).await?;
        Ok((code.unwrap_or_default(), ttl))
    }
    pub async fn set_code<T: ValidCodeData>(
        &self,
        tag: &String,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<(String, usize)> {
        if tag.len() > 255 {
            return Err(ValidCodeError::Tag(format!("tag over length:{}", tag)));
        }
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let code: Option<String> = redis.get(save_key.as_str()).await?;
        let out_code = valid_code_builder
            .get_code(&mut redis, &code, &self.prefix, tag)
            .await?;
        tracing::debug!(
            "valid-code data [ prefix: {} tag: {} code: {} ]",
            &self.prefix,
            &tag,
            out_code
        );
        if code.is_none() || out_code != code.unwrap_or_default() {
            redis.set(save_key.as_str(), out_code.clone()).await?;
        }
        let save_time = valid_code_builder.save_time();
        if save_time > 0 {
            redis.expire(save_key.as_str(), save_time).await?;
        }
        Ok((out_code, save_time))
    }
    pub async fn delay_code<T: ValidCodeData>(
        &self,
        tag: &String,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<(String, usize)> {
        let (code, llt) = self.get_code(tag).await?;
        if code.is_empty() || llt <= 1 {
            return Err(ValidCodeError::DelayTimeout(ValidCodeCheckError {
                message: "code is timeout".to_string(),
                prefix: self.prefix.to_owned(),
            }));
        }
        let save_time = valid_code_builder.save_time();
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        if save_time > 0 {
            redis.expire(save_key.as_str(), save_time).await?;
        }
        Ok((code, save_time))
    }
    pub async fn clear_code<T: ValidCodeData>(
        &self,
        tag: &String,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<()> {
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        redis.del(save_key).await?;
        valid_code_builder
            .clear_code(&mut redis, &self.prefix, tag)
            .await?;
        Ok(())
    }
    pub async fn check_code(&self, tag: &String, code: &String) -> ValidCodeResult<()> {
        let (s_code, _) = self.get_code(tag).await?;
        let c_code = code.trim();
        if c_code.is_empty() {
            return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                message: format!("your submit code [{}] is empty", code),
                prefix: self.prefix.to_owned(),
            }));
        }
        if if self.ignore_case {
            s_code.to_lowercase() != (*c_code).to_lowercase()
        } else {
            s_code != *c_code
        } {
            return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                message: format!("your submit code [{}] not match", code),
                prefix: self.prefix.to_owned(),
            }));
        }
        Ok(())
    }
}
