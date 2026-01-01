use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

#[derive(Default)]
pub struct ValidUrl<T: Display> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Display> ValidRule for ValidUrl<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        let regip = Regex::new(
            r"^http(s)?://[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}(:[\d]{1,5})?(/[^\s]*)?$$",
        )
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        let regdomain = Regex::new(
            r"^http(s)?://[0-9a-zA-Z]{0,1}[0-9a-zA-Z-]*(\.[0-9a-zA-Z-]*)*(\.[0-9a-zA-Z]*)+(:[\d]{1,5})?(/[^\s]*)?$$",
        )
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        if !regip.is_match(&data) && !regdomain.is_match(&data) {
            return Err(ValidRuleError::new(fluent_message!("valid-not-url",{
                "data":data,
            })));
        }
        Ok(())
    }
}
