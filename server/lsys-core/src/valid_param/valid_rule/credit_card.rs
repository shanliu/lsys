use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

#[derive(Default)]
pub struct ValidCreditCard<T: Display> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Display> ValidRule for ValidCreditCard<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        // Remove all non-digit characters
        let digits = data.to_string();

        // Credit card numbers are typically between 13 and 19 digits
        if digits.len() < 13 || digits.len() > 19 {
            return Err(ValidRuleError::new(fluent_message!(
                "valid-china-credit-card",{"data":digits}
            )));
        }

        // Luhn algorithm for validation
        let mut sum = 0;
        let mut alternate = false;
        for c in digits.chars().rev() {
            let mut n = match c.to_digit(10) {
                Some(d) => d,
                None => {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-china-credit-card",{"data":digits}
                    )));
                }
            };
            if alternate {
                n *= 2;
                if n > 9 {
                    n -= 9;
                }
            }
            sum += n;
            alternate = !alternate;
        }
        if sum % 10 != 0 {
            return Err(ValidRuleError::new(fluent_message!(
                "valid-china-credit-card",{"data":digits}
            )));
        }
        Ok(())
    }
}
