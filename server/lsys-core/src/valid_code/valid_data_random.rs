use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Connection};

use crate::{rand_str, RandType};

use super::{ValidCodeData, ValidCodeResult};

const CODE_CHANGE_KEY: &str = "valid-change";

pub struct ValidCodeDataRandom {
    pub save_time: isize,     //校验码有效时间
    pub duration_time: isize, //校验码持续不变时间
}
impl ValidCodeDataRandom {
    pub fn new(save_time: isize, duration_time: isize) -> Self {
        Self {
            save_time,
            duration_time,
        }
    }
}

#[async_trait]
impl ValidCodeData for ValidCodeDataRandom {
    async fn create_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        old_code: Option<&'t str>,
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<String> {
        let change_key = CODE_CHANGE_KEY.to_owned() + prefix + tag;
        if self.duration_time > 0 {
            if let Some(old_code) = old_code {
                if !old_code.trim().is_empty() {
                    let change_code: Option<String> = redis.get(change_key.as_str()).await?;
                    if change_code.unwrap_or_default() == old_code {
                        //一定时间内不变
                        return Ok(old_code.to_string());
                    }
                }
            }
        }
        let out_code = rand_str(RandType::Number, 6);
        let _: () = redis.set(change_key.as_str(), out_code.clone()).await?;
        let _: () = redis
            .expire(change_key.as_str(), self.duration_time as i64)
            .await?;
        Ok(out_code)
    }
    async fn destroy_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<()> {
        if self.duration_time > 0 {
            let change_key = CODE_CHANGE_KEY.to_owned() + prefix + tag;
            let _: () = redis.del(change_key).await?;
        }
        Ok(())
    }
    fn save_time(&self) -> usize {
        //保存时间必须比持续不变的时间长
        if self.save_time < self.duration_time {
            return self.duration_time as usize;
        }
        self.save_time as usize
    }
}
