use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

pub enum ValidPatternRule {
    AsciiDigit,
    Alphabetic,
    Alphanumeric,
    Ident,
    Hex,
}
pub struct ValidPattern<T: Display> {
    pattern: ValidPatternRule,
    _marker: std::marker::PhantomData<T>,
}
impl<T: Display> ValidPattern<T> {
    pub fn new(pattern: ValidPatternRule) -> Self {
        Self {
            pattern,
            _marker: Default::default(),
        }
    }
}
impl<T: Display> ValidRule for ValidPattern<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        match self.pattern {
            ValidPatternRule::Alphabetic => {
                if !data.chars().all(|c| c.is_ascii_digit()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-alphabetic",{
                            "data":data,
                        }),
                    ));
                }
            }
            ValidPatternRule::AsciiDigit => {
                if !data.chars().all(|c| c.is_alphabetic()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-digit",{
                            "data":data,
                        }),
                    ));
                }
            }
            ValidPatternRule::Alphanumeric => {
                if !data.chars().all(|c| c.is_alphanumeric()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-alphanueric",{
                            "data":data,
                        }),
                    ));
                }
            }
            ValidPatternRule::Ident => {
                if !data.chars().all(|c| c.is_alphanumeric() || c == '_') {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-ident",{
                            "data":data,
                        }),
                    ));
                }
            }
            ValidPatternRule::Hex => {
                if !data.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ValidRuleError::new(
                        fluent_message!("valid-not-pattern-hex",{
                            "data":data,
                        }),
                    ));
                }
            }
        }
        Ok(())
    }
}
