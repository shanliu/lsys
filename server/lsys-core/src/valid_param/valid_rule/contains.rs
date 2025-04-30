use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

pub struct ValidContains<'t, T: PartialEq + Display>(pub &'t [T]);
impl<T: PartialEq + Display> ValidRule for ValidContains<'_, T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        if self.0.contains(data) {
            return Err(ValidRuleError::new(fluent_message!("valid-not-contains",{
                "data":data,
                "data_list":self.0.iter().map(|e|e.to_string()).collect::<Vec<String>>().join(",")
            })));
        }
        Ok(())
    }
}
