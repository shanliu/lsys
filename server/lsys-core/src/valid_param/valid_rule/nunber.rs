use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::default::Default;
use std::fmt::Display;
use std::marker::PhantomData;

pub trait ValidNumberRange: PartialOrd + Copy + Default {
    /// 返回该类型的最大值
    fn max_value() -> Self;
    fn min_value() -> Self;
}

// 为常用数字类型实现 ValidNumberRange
macro_rules! impl_number_range {
    ($($t:ty),*) => {
        $(
            impl ValidNumberRange for $t {
                fn max_value() -> Self {
                    <$t>::MAX
                }
                fn min_value() -> Self {
                    <$t>::MIN
                }
            }
        )*
    };
}
impl_number_range!(u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64);

pub struct ValidNumber<T: ValidNumberRange> {
    min: T,
    max: T,
    _marker: PhantomData<T>,
}

impl<T: ValidNumberRange> ValidNumber<T> {
    pub fn id() -> ValidNumber<u64> {
        ValidNumber {
            min: 0,
            max: u64::MAX,
            _marker: PhantomData,
        }
    }
    pub fn range(min: T, max: T) -> Self {
        Self {
            min,
            max,
            _marker: PhantomData,
        }
    }
    pub fn eq(eq: T) -> Self {
        Self {
            min: eq,
            max: eq,
            _marker: PhantomData,
        }
    }
    pub fn min(min: T) -> Self {
        Self {
            min,
            max: T::max_value(),
            _marker: PhantomData,
        }
    }
    pub fn max(max: T) -> Self {
        Self {
            min: T::min_value(),
            max,
            _marker: PhantomData,
        }
    }
}

impl<T: ValidNumberRange> ValidRule for ValidNumber<T>
where
    T: PartialOrd + Copy + Display,
{
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        if self.min == self.max {
            if *data != self.min {
                return Err(ValidRuleError::new(fluent_message!(
                    "valid-not-number-equal",
                    {
                        "data": data,
                        "eq": self.min,
                    }
                )));
            }
        } else if *data < self.min || *data > self.max {
            return Err(ValidRuleError::new(fluent_message!(
                "valid-not-number-range",
                {
                    "data": data,
                    "min": self.min,
                    "max": self.max
                }

            )));
        }

        Ok(())
    }
}
