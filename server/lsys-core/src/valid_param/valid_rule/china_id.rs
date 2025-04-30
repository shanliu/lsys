use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

#[derive(Default)]
pub struct ValidChinaID<T: Display> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Display> ValidRule for ValidChinaID<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let id_str = data.to_string();
        let re = Regex::new(
            r"^[1-9]\d{5}(18|19|20)\d{2}((0[1-9])|(1[0-2]))(([0-2][1-9])|10|20|30|31)\d{3}[0-9Xx]$",
        )
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        if !re.is_match(&id_str) {
            return Err(ValidRuleError::new(fluent_message!(
                "valid-not-china-id",
                {
                "data": data,
                }
            )));
        }

        // 校验身份证有效性
        let factors = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
        let parity: [i32; 11] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let mut sum = 0;
        let id_str_chars: Vec<char> = id_str.chars().collect();
        for i in 0..17 {
            let digit = match id_str_chars[i].to_digit(10) {
                Some(t) => t as i32,
                None => {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-china-id",
                        {
                        "data": data,
                        }
                    )))
                }
            };
            sum += digit * factors[i];
        }

        let mod_result = sum % 11;
        let check_digit = match id_str_chars[17] {
            '0'..='9' => match id_str_chars[17].to_digit(10) {
                Some(t) => t as i32,
                None => {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-china-id",
                        {
                        "data": data,
                        }
                    )))
                }
            },
            'X' | 'x' => 10,
            _ => -1,
        };

        if check_digit != parity[(11 - mod_result) as usize] {
            return Err(ValidRuleError::new(fluent_message!(
                "valid-not-china-id-match",
                {
                "data": data,
                }
            )));
        }
        Ok(())
    }
}
