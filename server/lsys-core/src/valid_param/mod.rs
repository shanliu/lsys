// 校验码封装
mod result;
mod valid_rule;
pub use result::*;
pub use valid_rule::*;

#[derive(Default)]
pub struct ValidParamCheck<'a, D> {
    val: Vec<Box<dyn ValidRule<T = D> + 'a>>,
}

impl<'a, D> ValidParamCheck<'a, D> {
    pub fn add_rule<R: ValidRule<T = D> + 'a>(mut self, rule: R) -> Self {
        self.val.push(Box::new(rule));
        self
    }
}

#[derive(Default)]
pub struct ValidParam {
    val: Vec<(String, ValidRuleError)>,
}

impl ValidParam {
    pub fn add<D>(mut self, name: &str, data: &D, check: &ValidParamCheck<'_, D>) -> Self {
        for tmp in check.val.iter() {
            if let Err(err) = tmp.check(data) {
                self.val.push((name.to_string(), err));
            }
        }
        self
    }
    pub fn clear(mut self) -> Self {
        self.val = vec![];
        self
    }
    pub fn check(self) -> Result<(), ValidError> {
        if self.val.is_empty() {
            return Ok(());
        }
        Err(ValidError::new(self.val))
    }
}

#[test]
fn test_valid_param() {
    check_dome("kk", 0, vec!["aaa"]).unwrap();
    fn check_dome(email: &str, b: i32, mobile_list: Vec<&str>) -> Result<(), ValidError> {
        let mut valid = ValidParam::default()
            .add(
                "email",
                &email,
                &ValidParamCheck::default().add_rule(ValidEmail::default()),
            )
            .add(
                "mobile",
                &mobile_list,
                &ValidParamCheck::default().add_rule(ValidNotEmpty::default()),
            )
            .add(
                "status",
                &b,
                &ValidParamCheck::default().add_rule(ValidContains(&[1, 2, 3])),
            );
        for mobile in mobile_list {
            valid = valid.add(
                "mobile",
                &mobile,
                &ValidParamCheck::default().add_rule(ValidMobile::default()),
            );
        }
        valid.check()?;
        Ok(())
    }
}
