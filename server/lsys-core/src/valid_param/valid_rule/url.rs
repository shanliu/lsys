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
        let reg = Regex::new(r"^(https?://)?([a-zA-Z0-9-]+\.)+[a-zA-Z]{2,}(:\d+)?(/[^\s]*)?$")
            .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        if !reg.is_match(&data) {
            return Err(ValidRuleError::new(fluent_message!("valid-not-url",{
                "data":data,
            })));
        }
        Ok(())
    }
}
