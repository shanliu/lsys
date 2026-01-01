use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;
struct ValidStrMatchPhantom<T: Display>(std::marker::PhantomData<T>);
pub enum ValidStrMatch<'t, T: Display> {
    StartWith(&'t str),
    EndWith(&'t str),
    StartNotWith(&'t str),
    EndNotWith(&'t str),
    Contains(&'t str),
    NotContains(&'t str),
    #[doc(hidden)]
    #[allow(private_interfaces)]
    Phantom(ValidStrMatchPhantom<T>),
}

impl<T: Display> ValidRule for ValidStrMatch<'_, T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        match self {
            Self::StartWith(mstr) => {
                if !data.starts_with(*mstr) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-str-match-start-with",{"mstr":mstr,}
                    )));
                }
            }
            Self::EndWith(mstr) => {
                if !data.ends_with(*mstr) {
                    return Err(ValidRuleError::new(fluent_message!(
                       "valid-not-str-match-end-with",{"mstr":mstr,}
                    )));
                }
            }
            Self::StartNotWith(mstr) => {
                if data.starts_with(*mstr) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-str-match-start-not-with",{"mstr":mstr,}
                    )));
                }
            }
            Self::EndNotWith(mstr) => {
                if data.ends_with(*mstr) {
                    return Err(ValidRuleError::new(fluent_message!(
                       "valid-not-str-match-end-not-with",{"mstr":mstr,}
                    )));
                }
            }
            Self::Contains(mstr) => {
                if !data.contains(*mstr) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-str-match-contains",{"mstr":mstr,}
                    )));
                }
            }
            Self::NotContains(mstr) => {
                if data.contains(*mstr) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-str-match-not-contains",{"mstr":mstr,}
                    )));
                }
            }
            Self::Phantom(_) => {
                unreachable!("marker type unreachable");
            }
        }
        Ok(())
    }
}
