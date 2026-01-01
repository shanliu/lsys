// 校验码封装
mod result;
mod valid_data_random;

use async_trait::async_trait;
use deadpool_redis::{redis::AsyncCommands, Connection};
use log::debug;
pub use result::*;
use serde::{Deserialize, Serialize};
pub use valid_data_random::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidCodeInfo {
    pub code: String,
    pub check: u32,
}

use crate::{fluent_message, valid_key, ValidParam, ValidParamCheck, ValidPattern, ValidStrlen};
const CODE_SAVE_KEY: &str = "valid-save";

pub struct ValidCode {
    max_try: Option<u32>,
    prefix: String,
    ignore_case: bool,
    redis: deadpool_redis::Pool,
}

impl ValidCode {
    pub fn new(
        redis: deadpool_redis::Pool,
        prefix: &str,
        ignore_case: bool,
        max_try: Option<u32>,
    ) -> ValidCode {
        ValidCode {
            max_try,
            prefix: prefix.to_string(),
            redis,
            ignore_case,
        }
    }
    async fn tag_param_valid(&self, tag: &str) -> ValidCodeResult<()> {
        ValidParam::default()
            .add(
                valid_key!("tag"),
                &tag,
                &ValidParamCheck::default()
                    .add_rule(ValidPattern::NotFormat)
                    .add_rule(ValidStrlen::range(1, 255)),
            )
            .check()?;
        Ok(())
    }
    async fn get_code_data(&self, tag: &str) -> ValidCodeResult<(Option<ValidCodeInfo>, usize)> {
        self.tag_param_valid(tag).await?;
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let json_str: Option<String> = redis.get(save_key.as_str()).await?;
        let ttl = redis.ttl(save_key.as_str()).await?;

        let code_info = match json_str {
            Some(json) => {
                match serde_json::from_str::<ValidCodeInfo>(&json) {
                    Ok(info) => Some(info),
                    Err(_) => {
                        // 如果JSON解析失败，可能是旧格式的数据，尝试作为普通字符串处理
                        Some(ValidCodeInfo {
                            code: json,
                            check: 0,
                        })
                    }
                }
            }
            None => None,
        };

        Ok((code_info, ttl))
    }
}

#[async_trait]
pub trait ValidCodeData {
    //创建一个校验码
    async fn create_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        code: Option<&'t str>, //上一次生成的校验码
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<String>;
    //销毁当前已生成的校验码
    async fn destroy_code<'t>(
        &mut self,
        redis: &'t mut Connection,
        prefix: &'t str,
        tag: &'t str,
    ) -> ValidCodeResult<()>;
    //当前校验码的有效时间
    fn save_time(&self) -> usize;
}

impl ValidCode {
    //设置校验码
    pub async fn set_code<T: ValidCodeData>(
        &self,
        tag: &str,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<(String, usize)> {
        self.tag_param_valid(tag).await?;
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let (code_info, _) = self.get_code_data(tag).await?;
        let existing_code = code_info.as_ref().map(|info| info.code.as_str());

        let out_code = valid_code_builder
            .create_code(&mut redis, existing_code, &self.prefix, tag)
            .await?;

        tracing::debug!(
            "valid-code data [ prefix: {} tag: {} code: {} ]",
            &self.prefix,
            &tag,
            out_code
        );

        // 创建新的验证码信息结构
        let code_info = ValidCodeInfo {
            code: out_code.clone(),
            check: 0, // 初始检测次数为0
        };

        // 序列化为JSON字符串
        let json_str = serde_json::to_string(&code_info).map_err(ValidCodeError::Serialize)?;

        let _: () = redis.set(save_key.as_str(), json_str).await?;
        let save_time = valid_code_builder.save_time();
        if save_time > 0 {
            let _: () = redis.expire(save_key.as_str(), save_time as i64).await?;
        }
        Ok((out_code, save_time))
    }
    //销毁当前校验码
    pub async fn destroy_code<T: ValidCodeData>(
        &self,
        tag: &str,
        valid_code_builder: &mut T,
    ) -> ValidCodeResult<()> {
        self.tag_param_valid(tag).await?;
        let save_key = CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + tag;
        let mut redis = self.redis.get().await?;
        let _: () = redis.del(save_key).await?;
        valid_code_builder
            .destroy_code(&mut redis, &self.prefix, tag)
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
    //检查当前校验码是否正确
    pub async fn check_code(&self, check_data: &CheckCodeData<'_>) -> ValidCodeResult<()> {
        let (inner_code_info, _) = self.get_code_data(check_data.tag.trim()).await?;
        let c_code = check_data.code.trim();

        if c_code.is_empty() {
            return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                message: fluent_message!("valid-code-submit-empty"),
                prefix: self.prefix.to_owned(),
            }));
        }

        match inner_code_info {
            Some(mut code_info) => {
                if let Some(max_try) = self.max_try {
                    if code_info.check >= max_try {
                        return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                            message: fluent_message!("valid-code-max-try",{//format!("your submit code [{}] not match", code)
                                "code":check_data.code
                            }),
                            prefix: self.prefix.to_owned(),
                        }));
                    }
                }
                let s_code = &code_info.code;
                let is_match = if self.ignore_case {
                    s_code.to_lowercase() == c_code.to_lowercase()
                } else {
                    s_code == c_code
                };

                // 无论匹配与否，都要增加检测次数
                code_info.check += 1;

                // 更新Redis中的JSON数据
                let save_key =
                    CODE_SAVE_KEY.to_owned() + self.prefix.as_str() + check_data.tag.trim();
                let json_str =
                    serde_json::to_string(&code_info).map_err(ValidCodeError::Serialize)?;

                let mut redis = self.redis.get().await?;
                let _: () = redis.set(save_key.as_str(), json_str).await?;

                if !is_match {
                    debug!(
                        "valid-not-match[case:{}]:{}!={} (check count: {})",
                        self.ignore_case, s_code, c_code, code_info.check
                    );
                    return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                        message: fluent_message!("valid-code-not-match",{//format!("your submit code [{}] not match", code)
                            "code":check_data.code
                        }),
                        prefix: self.prefix.to_owned(),
                    }));
                }
            }
            None => {
                return Err(ValidCodeError::NotMatch(ValidCodeCheckError {
                    message: fluent_message!("valid-code-not-found",{//format!("your submit code [{}] not match", code)
                        "code":check_data.code
                    }),
                    prefix: self.prefix.to_owned(),
                }));
            }
        }
        Ok(())
    }
}
