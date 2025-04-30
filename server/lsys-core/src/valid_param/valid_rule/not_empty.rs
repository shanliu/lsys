use crate::{fluent_message, ValidRuleError};

use super::ValidRule;

pub trait ValidIsNotEmpty {
    fn is_not_empty(&self) -> bool;
}

#[derive(Default)]
pub struct ValidNotEmpty<S: ValidIsNotEmpty> {
    _marker: std::marker::PhantomData<S>,
}
impl<S: ValidIsNotEmpty> ValidRule for ValidNotEmpty<S> {
    type T = S;
    fn check(&self, data: &Self::T) -> Result<(), ValidRuleError> {
        if data.is_not_empty() {
            return Ok(());
        }
        Err(ValidRuleError::new(fluent_message!("valid-not-empty")))
    }
}

impl ValidIsNotEmpty for &str {
    fn is_not_empty(&self) -> bool {
        !self.trim().is_empty()
    }
}
impl ValidIsNotEmpty for String {
    fn is_not_empty(&self) -> bool {
        !self.trim().is_empty()
    }
}
impl<T> ValidIsNotEmpty for Vec<T> {
    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}
impl<T> ValidIsNotEmpty for &Vec<T> {
    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}
impl<T> ValidIsNotEmpty for &[T] {
    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }
}
