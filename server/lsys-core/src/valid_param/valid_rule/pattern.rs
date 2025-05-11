use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

struct ValidPatternPhantom<T: Display>(std::marker::PhantomData<T>);
pub enum ValidPattern<T: Display> {
    Alphabetic,   //字母
    Alphanumeric, //字母数字
    Numeric,      //0-10数字
    NotFormat,
    Ident, //内部用标识符,需跟字符过滤[StringClear::Ident]保持一致,对用户标识(user_data),应用标识(client_id)等进行统一的验证
    Hex,

    #[doc(hidden)]
    #[allow(private_interfaces)]
    Phantom(ValidPatternPhantom<T>),
}

impl<T: Display> ValidRule for ValidPattern<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        match self {
            Self::NotFormat => {
                if data
                    .chars()
                    .any(|c| ['\t', '\0', '\\', '\n', '\r'].contains(&c))
                    || data.trim().len() != data.len()
                    || data.contains("  ")
                {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-pattern-not-format"
                    )));
                }
            }
            Self::Numeric => {
                if !data.chars().all(|c| c.is_ascii_digit()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-numeric",{
                            "data":data,
                        }),
                    ));
                }
            }
            Self::Alphabetic => {
                if !data.chars().all(|c| c.is_alphabetic()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-alphabetic",{
                            "data":data,
                        }),
                    ));
                }
            }
            Self::Alphanumeric => {
                if !data.chars().all(|c| c.is_alphanumeric()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-alphanueric",{
                            "data":data,
                        }),
                    ));
                }
            }
            Self::Ident => {
                //数字，字母 - _ . 组成，开头或结尾不能是 - _ .
                if let Some(c) = data.chars().next() {
                    if c == '-' || c == '_' || c == '.' {
                        return Err(ValidRuleError::new(
                            fluent_message!("valid-not-pattern-ident",{
                                "data":data,
                            }),
                        ));
                    }
                }
                if let Some(c) = data.chars().last() {
                    if c == '-' || c == '_' || c == '.' {
                        return Err(ValidRuleError::new(
                            fluent_message!("valid-not-pattern-ident",{
                                "data":data,
                            }),
                        ));
                    }
                }
                if !data
                    .chars()
                    .all(|c| c.is_alphanumeric() || c == '-' || c == '_' || c == '.')
                {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-ident",{
                            "data":data,
                        }),
                    ));
                }
            }
            Self::Hex => {
                if !data.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-hex",{
                            "data":data,
                        }),
                    ));
                }
            }
            Self::Phantom(_) => {
                unreachable!("marker type unreachable");
            }
        }
        Ok(())
    }
}
