use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

pub struct ValidStrlen<T: Display> {
    min: u64,
    max: u64,
    _marker: std::marker::PhantomData<T>,
}
impl<T: Display> ValidStrlen<T> {
    pub fn range(min: u64, max: u64) -> Self {
        Self {
            min,
            max,
            _marker: Default::default(),
        }
    }
    pub fn max(max: u64) -> Self {
        Self {
            min: 0,
            max,
            _marker: Default::default(),
        }
    }
    pub fn min(min: u64) -> Self {
        Self {
            min,
            max: u64::MAX,
            _marker: Default::default(),
        }
    }
    pub fn eq(eq: u64) -> Self {
        Self {
            min: eq,
            max: eq,
            _marker: Default::default(),
        }
    }
}
impl<T: Display> ValidRule for ValidStrlen<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        let len = data.chars().count() as u64;
        if self.max == self.min {
            if len != self.min {
                return Err(ValidRuleError::new(fluent_message!(
                    "valid-not-strlen-equal",
                    {
                        "len": len,
                        "eq": self.min
                    }
                )));
            }
        } else if len < self.min || len > self.max {
            return Err(ValidRuleError::new(fluent_message!(
                "valid-not-strlen-range",
                {
                    "len": len,
                    "min": self.min,
                    "max": self.max
                }
            )));
        }
        Ok(())
    }
}
