use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

pub enum ValidPasswordLevel {
    Low,
    Medium,
    Strong,
}
pub struct ValidPassword<T: Display> {
    level: ValidPasswordLevel,
    _marker: std::marker::PhantomData<T>,
}
impl<T: Display> ValidPassword<T> {
    pub fn new(level: ValidPasswordLevel) -> Self {
        Self {
            level,
            _marker: Default::default(),
        }
    }
}
impl<T: Display> ValidRule for ValidPassword<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let dt_str = data.to_string();
        match self.level {
            ValidPasswordLevel::Low => {
                if dt_str.len() < 6 || dt_str.contains([' ', '\t', '\r', '\n']) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-password-Low",{"len":6,}
                    )));
                }
            }
            ValidPasswordLevel::Medium => {
                if dt_str.len() < 6 || dt_str.contains([' ', '\t', '\r', '\n']) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-password-medium",{"len":6,}
                    )));
                }
                // 检查完全重复的模式，如 111111, aaaaaa
                let all_same = dt_str
                    .chars()
                    .next()
                    .is_some_and(|first| dt_str.chars().all(|c| c == first));

                // 检查数字的连续模式，如 123456, 654321
                let consecutive_numbers = dt_str.chars().all(|c| c.is_ascii_digit())
                    && dt_str
                        .as_bytes()
                        .windows(2)
                        .all(|w| (w[1] as i8 - w[0] as i8).abs() == 1);

                // 检查字母的连续模式，如 abcdef
                let consecutive_letters = dt_str.chars().all(|c| c.is_ascii_alphabetic())
                    && dt_str
                        .to_lowercase()
                        .as_bytes()
                        .windows(2)
                        .all(|w| (w[1] as i8 - w[0] as i8).abs() == 1);

                // 检查重复的双字符模式，如 112233
                let repeating_pairs = dt_str
                    .as_bytes()
                    .chunks(2)
                    .all(|chunk| chunk.len() == 2 && chunk[0] == chunk[1]);

                if all_same || consecutive_numbers || consecutive_letters || repeating_pairs {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-password-medium",{"len":6,}
                    )));
                }
            }
            ValidPasswordLevel::Strong => {
                if dt_str.len() < 8 || dt_str.contains([' ', '\t', '\r', '\n']) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-password-strong",{"len":8,}
                    )));
                }
                let has_letter = dt_str.chars().any(|c| c.is_ascii_alphabetic());
                let has_digit = dt_str.chars().any(|c| c.is_ascii_digit());
                let has_special = dt_str.chars().any(|c| !c.is_ascii_alphanumeric());
                if !has_letter || !has_digit || !has_special {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-password-strong",{"len":8,}
                    )));
                }
            }
        }
        Ok(())
    }
}
