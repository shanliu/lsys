use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

#[derive(Default)]
pub struct ValidChinaLicensePlate<T: Display> {
    _marker: std::marker::PhantomData<T>,
}

impl<T: Display> ValidRule for ValidChinaLicensePlate<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        let regular_pattern = Regex::new(r"^[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼使领][A-HJ-NP-Z][A-HJ-NP-Z0-9]{4}[A-HJ-NP-Z0-9挂学警港澳]$") 
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        let new_energy_pattern = Regex::new(r"^[京津沪渝冀豫云辽黑湘皖鲁新苏浙赣鄂桂甘晋蒙陕吉闽贵粤青藏川宁琼使领][A-HJ-NP-Z](?:(?:[DF][A-HJ-NP-Z0-9]{4})|(?:[0-9]{5}[DF]))$")
        .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
        if !regular_pattern.is_match(&data) && !new_energy_pattern.is_match(&data) {
            return Err(ValidRuleError::new(fluent_message!(
                "valid-china-license-plate",{"data":data}
            )));
        }
        Ok(())
    }
}
