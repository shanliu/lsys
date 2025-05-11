use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

struct ValidDateTimePhantom<T: Display>(std::marker::PhantomData<T>);
pub enum ValidDateTime<T: Display> {
    Date,
    Time,
    DateTime,
    DateTimeZone,
    #[doc(hidden)]
    #[allow(private_interfaces)]
    Phantom(ValidDateTimePhantom<T>),
}

impl<T: Display> ValidRule for ValidDateTime<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let dt_str = data.to_string();

        if !Regex::new(match self {
            Self::Phantom(_) => {
                unreachable!("marker type unreachable");
            }
            Self::Date => r"^\d{4}-\d{2}-\d{2}$",
            Self::Time => r"^\d{2}:\d{2}:\d{2}$",
            Self::DateTime => r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$",
            Self::DateTimeZone => r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}[+-]\d{2}:\d{2}$",
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
