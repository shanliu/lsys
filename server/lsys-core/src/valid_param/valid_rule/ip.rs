use regex::Regex;

use super::ValidRule;
use crate::{fluent_message, ValidRuleError};
use std::fmt::Display;

pub const VALID_IP_V4: u32 = 1 << 0;
pub const VALID_IP_V6: u32 = 1 << 1;
pub const VALID_IP: u32 = VALID_IP_V4 | VALID_IP_V6;

pub struct ValidIp<T: Display> {
    _marker: std::marker::PhantomData<T>,
    flag: u32,
}

impl<T: Display> Default for ValidIp<T> {
    fn default() -> Self {
        Self {
            _marker: Default::default(),
            flag: VALID_IP,
        }
    }
}
impl<T: Display> ValidIp<T> {
    pub fn new(flag: u32) -> Self {
        Self {
            _marker: Default::default(),
            flag,
        }
    }
}

impl<T: Display> ValidRule for ValidIp<T> {
    type T = T;
    fn check(&self, data: &T) -> Result<(), ValidRuleError> {
        let data = data.to_string();
        let mut match_data = vec![];
        if (self.flag & VALID_IP_V4) != 0 {
            let ipv4_regex = Regex::new(r"^((25[0-5]|2[0-4]\d|1\d{2}|[1-9]?\d)(\.|$)){4}$")
                .map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
            if ipv4_regex.is_match(&data) {
                match_data.push(true);
            }
        }
        if (self.flag & VALID_IP_V6) != 0 {
            let ipv6_regex = Regex::new(r"(?xi)
                    ^
                    (
                        ([0-9a-f]{1,4}:){7}[0-9a-f]{1,4} |                                 # 1:1:1:1:1:1:1:1
                        ([0-9a-f]{1,4}:){1,7}: |                                           # 1::                              1:2:3:4:5:6:7::
                        ([0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4} |                              # 1::8             1:2:3:4:5:6::8
                        ([0-9a-f]{1,4}:){1,5}(:[0-9a-f]{1,4}){1,2} |                       # 1::7:8           1:2:3:4:5::7:8
                        ([0-9a-f]{1,4}:){1,4}(:[0-9a-f]{1,4}){1,3} |                       # 1::6:7:8         1:2:3:4::6:7:8
                        ([0-9a-f]{1,4}:){1,3}(:[0-9a-f]{1,4}){1,4} |                       # 1::5:6:7:8       1:2:3::5:6:7:8
                        ([0-9a-f]{1,4}:){1,2}(:[0-9a-f]{1,4}){1,5} |                       # 1::4:5:6:7:8     1:2::4:5:6:7:8
                        [0-9a-f]{1,4}:((:[0-9a-f]{1,4}){1,6}) |                            # 1::3:4:5:6:7:8   1::3:4:5:6:7:8
                        :((:[0-9a-f]{1,4}){1,7}|:) |                                       # ::2:3:4:5:6:7:8  ::8              ::
                        fe80:(:[0-9a-f]{0,4}){0,4}%[0-9a-zA-Z]{1,} |                       # fe80::7:8%eth0   fe80::7:8%1
                        ::ffff:(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3} | # ::ffff:192.168.1.1
                        ([0-9a-f]{1,4}:){1,4}:(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3} # 1:2:3:4:5:6:7:8
                    )
                $").map_err(|e| ValidRuleError::new(fluent_message!("valid-regex-error", e)))?;
            if ipv6_regex.is_match(&data) {
                match_data.push(true);
            }
        }
        if match_data.is_empty() {
            return Err(ValidRuleError::new(fluent_message!("valid-not-ip", {
                "data":data
            })));
        }
        Ok(())
    }
}

#[test]
fn ccc() {
    let ipv6_regex = Regex::new(r"(?xi)
    ^
    (
        ([0-9a-f]{1,4}:){7}[0-9a-f]{1,4} |                                 # 1:1:1:1:1:1:1:1
        ([0-9a-f]{1,4}:){1,7}: |                                           # 1::                              1:2:3:4:5:6:7::
        ([0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4} |                              # 1::8             1:2:3:4:5:6::8
        ([0-9a-f]{1,4}:){1,5}(:[0-9a-f]{1,4}){1,2} |                       # 1::7:8           1:2:3:4:5::7:8
        ([0-9a-f]{1,4}:){1,4}(:[0-9a-f]{1,4}){1,3} |                       # 1::6:7:8         1:2:3:4::6:7:8
        ([0-9a-f]{1,4}:){1,3}(:[0-9a-f]{1,4}){1,4} |                       # 1::5:6:7:8       1:2:3::5:6:7:8
        ([0-9a-f]{1,4}:){1,2}(:[0-9a-f]{1,4}){1,5} |                       # 1::4:5:6:7:8     1:2::4:5:6:7:8
        [0-9a-f]{1,4}:((:[0-9a-f]{1,4}){1,6}) |                            # 1::3:4:5:6:7:8   1::3:4:5:6:7:8
        :((:[0-9a-f]{1,4}){1,7}|:) |                                       # ::2:3:4:5:6:7:8  ::8              ::
        fe80:(:[0-9a-f]{0,4}){0,4}%[0-9a-zA-Z]{1,} |                       # fe80::7:8%eth0   fe80::7:8%1
        ::ffff:(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3} | # ::ffff:192.168.1.1
        ([0-9a-f]{1,4}:){1,4}:(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)(\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3} # 1:2:3:4:5:6:7:8
    )
    $
").unwrap();
    println!("{:?}", ipv6_regex.is_match("fe80::e1bd:c78d:610f:3d03"));
}
