use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

struct ValidGithantom<T: Display>(std::marker::PhantomData<T>);
pub enum ValidGit<T: Display> {
    VersionHash,
    #[doc(hidden)]
    #[allow(private_interfaces)]
    Phantom(ValidGithantom<T>),
}

impl<T: Display> ValidRule for ValidGit<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let dt_str = data.to_string();
        match self {
            Self::Phantom(_) => {
                unreachable!("marker type unreachable");
            }
            ValidGit::VersionHash => {
                if dt_str.len() != 40 {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-git-version"
                    )));
                }
                if !dt_str.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(ValidRuleError::new(fluent_message!(
                        "valid-not-git-version"
                    )));
                }
            }
        }
        Ok(())
    }
}
