use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

pub enum ValidDateTimeRule {
    Date,
    Time,
    DateTime,
    DateTimeZone,
}
pub struct ValidDateTime<T: Display> {
    rule: ValidDateTimeRule,
    _marker: std::marker::PhantomData<T>,
}
impl<T: Display> ValidDateTime<T> {
    pub fn new(rule: ValidDateTimeRule) -> Self {
        Self {
            rule,
            _marker: Default::default(),
        }
    }
}
impl<T: Display> ValidRule for ValidDateTime<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let dt_str = data.to_string();

        if !Regex::new(match self.rule {
            ValidDateTimeRule::Date => r"^\d{4}-\d{2}-\d{2}$",
            ValidDateTimeRule::Time => r"^\d{2}:\d{2}:\d{2}$",
            ValidDateTimeRule::DateTime => r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$",
            ValidDateTimeRule::DateTimeZone => {
                r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}$"
            }
        })
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?
        .is_match(&dt_str)
        {
            return Err(ValidRuleError::new(
                fluent_message!("valid-not-datetime",{"data":dt_str,}),
            ));
        }
        Ok(())
    }
}
