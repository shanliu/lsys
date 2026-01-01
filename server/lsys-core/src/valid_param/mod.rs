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
#[derive(Clone, Debug)]
pub struct ValidRuleKey {
    pub name: String,
    pub crate_name: &'static str,
}

#[derive(Default, Clone)]
pub struct ValidParam {
    val: Vec<(ValidRuleKey, ValidRuleError)>,
}

impl ValidParam {
    pub fn add<D>(
        &mut self,
        key: ValidRuleKey,
        data: &D,
        check: &ValidParamCheck<'_, D>,
    ) -> &mut Self {
        for tmp in check.val.iter() {
            if let Err(err) = tmp.check(data) {
                self.val.push((key.clone(), err));
            }
        }
        self
    }
    pub fn clear(&mut self) -> &mut Self {
        self.val = vec![];
        self
    }
    pub fn check(&mut self) -> Result<(), ValidError> {
        let val = std::mem::take(&mut self.val);
        if val.is_empty() {
            return Ok(());
        }
        Err(ValidError::new(val))
    }
}

#[macro_export]
macro_rules! valid_key {
    ($name:literal) => {
        $crate::ValidRuleKey {
            name: $name.to_string(),
            crate_name: env!("CARGO_PKG_NAME"),
        }
    };
}

#[test]
fn test_valid_param() {
    check_dome("kk@qq.com", 1, vec!["13800138000"]).unwrap();
    fn check_dome(email: &str, b: i32, mobile_list: Vec<&str>) -> Result<(), ValidError> {
        let mut valid = ValidParam::default();
        valid
            .add(
                valid_key!("email"),
                &email,
                &ValidParamCheck::default().add_rule(ValidEmail::default()),
            )
            .add(
                valid_key!("mobile"),
                &mobile_list,
                &ValidParamCheck::default().add_rule(ValidNotEmpty::default()),
            )
            .add(
                valid_key!("status"),
                &b,
                &ValidParamCheck::default().add_rule(ValidContains(&[1, 2, 3])),
            );
        for mobile in mobile_list {
            valid.add(
                valid_key!("mobile"),
                &mobile,
                &ValidParamCheck::default().add_rule(ValidMobile::default()),
            );
        }
        valid.check()?;
        Ok(())
    }
}
