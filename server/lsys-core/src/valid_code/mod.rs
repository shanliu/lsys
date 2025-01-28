use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Connection};

mod result;
pub use result::*;

use crate::{fluent_message, rand_str, RandType};
const CODE_SAVE_KEY: &str = "valid-save";

pub struct ValidCode {
    prefix: String,
    ignore_case: bool,
    redis: deadpool_redis::Pool,
}
#[async_trait]
pub trait ValidCodeData {
    async fn get_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        code: Option<&'t str>,
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
        Ok(rand_str(RandType::Number, 6))
        // const BASE_STR: &str = "0123456789";
        // let mut rng = &mut rand::thread_rng();
        // Ok(String::from_utf8(
        //     BASE_STR
        //         .as_bytes()
        //         .choose_multiple(&mut rng, 6)
        //         .cloned()
        //         .collect(),
        // )?)
    }
}
#[async_trait]
impl ValidCodeData for ValidCodeDataRandom {
    async fn get_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        code: Option<&'t str>,
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
                old_code.to_string()
            };

        if self.save_time() > 0 && duration_time > 0 {
            let _: () = redis.set(change_key.as_str(), out_code.clone()).await?;
            let _: () = redis
                .expire(change_key.as_str(), duration_time as i64)
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
        let _: () = redis.del(change_key).await?;
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
    pub fn new(redis: deadpool_redis::Pool, prefix: &str, ignore_case: bool) -> ValidCode {
        ValidCode {
            prefix: prefix.to_string(),
            redis,
            ignore_case,
        }
    }
    pub async fn get_code(&self, tag: &str) -> ValidCodeResult<(String, usize)> {
        if tag.len() > 255 {
            return Err(ValidCodeError::Tag(fluent_message!("valid-code-tag-len",{
                "tag":tag,
                "max":255
            })));
        }
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let code: Option<String> = redis.get(save_key.as_str()).await?;
        let ttl = redis.ttl(save_key.as_str()).await?;
        Ok((code.unwrap_or_default(), ttl))
    }
    pub async fn set_code<T: ValidCodeData>(
        &self,
        tag: &str,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<(String, usize)> {
        if tag.len() > 255 {
            return Err(ValidCodeError::Tag(fluent_message!("valid-code-tag-len",{
                "tag":tag,
                "max":255
            })));
        }
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let code: Option<String> = redis.get(save_key.as_str()).await?;
        let out_code = valid_code_builder
            .get_code(&mut redis, code.as_deref(), &self.prefix, tag)
            .await?;
        tracing::debug!(
            "valid-code data [ prefix: {} tag: {} code: {} ]",
            &self.prefix,
            &tag,
            out_code
        );
        if code.is_none() || out_code != code.unwrap_or_default() {
            let _: () = redis.set(save_key.as_str(), out_code.clone()).await?;
        }
        let save_time = valid_code_builder.save_time();
        if save_time > 0 {
            let _: () = redis.expire(save_key.as_str(), save_time as i64).await?;
        }
        Ok((out_code, save_time))
    }
    pub async fn delay_code<T: ValidCodeData>(
        &self,
        tag: &str,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<(String, usize)> {
        let (code, llt) = self.get_code(tag).await?;
        if code.is_empty() || llt <= 1 {
            return Err(ValidCodeError::DelayTimeout(ValidCodeCheckError {
                message: fluent_message!("valid-code-timeout"), //"code is timeout".to_string()
                prefix: self.prefix.to_owned(),
            }));
        }
        let save_time = valid_code_builder.save_time();
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        if save_time > 0 {
            let _: () = redis.expire(save_key.as_str(), save_time as i64).await?;
        }
        Ok((code, save_time))
    }
    pub async fn clear_code<T: ValidCodeData>(
        &self,
        tag: &str,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<()> {
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let _: () = redis.del(save_key).await?;
        valid_code_builder
            .clear_code(&mut redis, &self.prefix, tag)
            .await?;
        Ok(())
    }
}

pub struct CheckCodeData<'t> {
    pub tag: &'t str,
    pub code: &'t str,
}

impl CheckCodeData<'_> {
    pub fn new<'t>(tag: &'t str, code: &'t str) -> CheckCodeData<'t> {
        CheckCodeData { tag, code }
    }
}

impl ValidCode {
    pub async fn check_code(&self, check_data: &CheckCodeData<'_>) -> ValidCodeResult<()> {
        let (s_code, _) = self.get_code(check_data.tag.trim()).await?;
        let c_code = check_data.code.trim();
        if c_code.is_empty() {
            return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                message: fluent_message!("valid-code-submit-empty"),
                prefix: self.prefix.to_owned(),
            }));
        }
        if if self.ignore_case {
            s_code.to_lowercase() != (*c_code).to_lowercase()
        } else {
            s_code != *c_code
        } {
            return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                message: fluent_message!("valid-code-not-match",{//format!("your submit code [{}] not match", code)
                    "code":check_data.code
                }),
                prefix: self.prefix.to_owned(),
            }));
        }
        Ok(())
    }
}
