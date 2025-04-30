use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

#[derive(Default)]
pub struct ValidMobile<T: Display> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Display> ValidRule for ValidMobile<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let mobile = data.to_string();
        let mut is_china_mobile = false;
        let mobile: &str = if mobile.len() == 11 {
            is_china_mobile = true;
            &mobile
        } else if mobile.chars().count() == 13 {
            if mobile.chars().take(2).collect::<String>() == *"86" {
                is_china_mobile = true;
            }
            let mut chars = mobile.chars();
            chars.nth(1);
            chars.as_str()
        } else if mobile.chars().count() == 14 {
            let pf = mobile.chars().take(3).collect::<String>();
            if ["+86", "086"].contains(&pf.as_str()) {
                is_china_mobile = true;
            }
            let mut chars = mobile.chars();
            chars.nth(2);
            chars.as_str()
        } else {
            return Err(ValidRuleError::new(fluent_message!("valid-not-mobile",{
                "data":data,
            })));
        };
        let mobile_re = if is_china_mobile {
            Regex::new(r"^1[3-9][\d]{9}+$")
        } else {
            Regex::new(r"^+?[\d]+[-]?[\d]+$")
        }
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        if !mobile_re.is_match(mobile) {
            return Err(ValidRuleError::new(fluent_message!("valid-not-mobile",{
                "data":data,
            })));
        }
        Ok(())
    }
}
