use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

pub enum ValidGitType {
    VersionHash,
}
pub struct ValidGit<T: Display> {
    level: ValidGitType,
    _marker: std::marker::PhantomData<T>,
}
impl<T: Display> ValidGit<T> {
    pub fn new(level: ValidGitType) -> Self {
        Self {
            level,
            _marker: Default::default(),
        }
    }
}
impl<T: Display> ValidRule for ValidGit<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let dt_str = data.to_string();
        match self.level {
            ValidGitType::VersionHash => {
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
