use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

#[derive(Default)]
pub struct ValidDomain<T: Display> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Display> ValidRule for ValidDomain<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        let ipre = Regex::new(r"^[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}\.[\d]{1,3}(:[\d]{1,5})?$")
            .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        let dre = Regex::new(
            r"^[0-9a-zA-Z]{0,1}[0-9a-zA-Z-]*(\.[0-9a-zA-Z-]*)*(\.[0-9a-zA-Z]*)+(:[\d]{1,5})?$",
        )
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        if !ipre.is_match(&data) && !dre.is_match(&data) {
            return Err(ValidRuleError::new(fluent_message!("valid-not-domain",{
                "data":data
            })));
        }
        Ok(())
    }
}
