use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;
struct ValidColorPhantom<T: Display>(std::marker::PhantomData<T>);
pub enum ValidColor<T: Display> {
    RGB,
    RGBA,
    #[doc(hidden)]
    #[allow(private_interfaces)]
    Phantom(ValidColorPhantom<T>),
}

impl<T: Display> ValidRule for ValidColor<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let dt_str = data.to_string();
        match self {
            Self::Phantom(_) => {
                unreachable!("marker type unreachable");
            }
            Self::RGB => {
                if !dt_str.starts_with('#')
                    || dt_str.len() != 7
                    || !dt_str[1..].chars().all(|c| c.is_ascii_hexdigit())
                {
                    return Err(ValidRuleError::new(fluent_message!("valid-not-rgb")));
                }
            }
            Self::RGBA => {
                if !dt_str.starts_with('#')
                    || dt_str.len() != 9
                    || !dt_str[1..].chars().all(|c| c.is_ascii_hexdigit())
                {
                    return Err(ValidRuleError::new(fluent_message!("valid-not-rgba")));
                }
            }
        }
        Ok(())
    }
}
