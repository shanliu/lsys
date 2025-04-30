use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use regex::Regex;
use std::fmt::Display;

#[derive(Default)]
pub struct ValidEmail<T: Display> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Display> ValidRule for ValidEmail<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let re = Regex::new(r"^[A-Za-z0-9\u4e00-\u9fa5\.\-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$")
            .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        if !re.is_match(&data.to_string()) {
            return Err(ValidRuleError::new(fluent_message!("valid-not-email",{
                "data":data
            })));
        }
        Ok(())
    }
}
