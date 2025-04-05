use regex::Regex;

use super::{AccountError, AccountResult};

pub fn check_email(email: &str) -> AccountResult<()> {
    let re = Regex::new(r"^[A-Za-z0-9\u4e00-\u9fa5\.\-]+@[a-zA-Z0-9_-]+(\.[a-zA-Z0-9_-]+)+$")
        .map_err(|e| {
            AccountError::Param(lsys_core::fluent_message!("rule-error",e))
        })?;
    if !re.is_match(email) {
        return Err(AccountError::Param(
            lsys_core::fluent_message!("auth-email-not-match",{
                    "mail":email,
                }
            ),
        )); //"submit email is invalid"
    }
    Ok(())
}

pub fn check_mobile(area: &str, mobile: &str) -> AccountResult<()> {
    if !area.is_empty() {
        let area_re = Regex::new(r"^[\+\d]{0,1}[\d]{0,3}$").map_err(|e| {
            AccountError::Param(lsys_core::fluent_message!("rule-error",e))
        })?;
        if !area_re.is_match(area) {
            return Err(AccountError::Param(
                lsys_core::fluent_message!("auth-mobile-area-error",
                    {
                        "area":area,
                    }
                ),
            )); //"submit area code is invalid"
        }
    }
    let mobile_re = Regex::new(r"^[\d]{0,1}[\-\d]{4,12}$").map_err(|e| {
        AccountError::Param(lsys_core::fluent_message!("rule-error",e))
    })?;
    if !mobile_re.is_match(mobile) {
        return Err(AccountError::Param(
            lsys_core::fluent_message!("auth-mobile-error",
                {
                    "mobile":mobile,
                    "msg":"not match"
                }

            ),
        )); //"submit mobile is invalid"
    }
    Ok(())
}

#[test]
fn test_phone() {
    let re = Regex::new(r"^[\+\d]{0,1}[\d]{0,3}$").unwrap();
    assert!(re.is_match("111"));
    let re = Regex::new(r"^[\d]{0,1}[\-\d]{4,12}$").unwrap();
    assert!(re.is_match("11111"));
}
