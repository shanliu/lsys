//RBAC中资源相关实现
use lsys_core::fluent_message;
use std::str::FromStr;
use std::vec;

use crate::dao::res::RbacRes;
use crate::dao::result::RbacError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ResCacheKey {
    pub res_type: String, //资源类型
    pub res_data: String, //资源数据
    pub user_id: u64,     //资源用户ID
    pub app_id: u64,
}

impl std::fmt::Display for ResCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}-{}-{}-{}",
            self.user_id, self.app_id, self.res_type, self.res_data
        )
    }
}

impl FromStr for ResCacheKey {
    type Err = RbacError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut token_split = s.split('-');
        let user_id = token_split.next().ok_or_else(|| {
            RbacError::System(fluent_message!("rbac-parse-res-str-fail",{
                "token":s
            }))
        })?;
        let user_id = user_id.parse::<u64>().map_err(|e| {
            RbacError::System(fluent_message!("rbac-parse-res-str-fail",{
                "token":s,
                "msg":e
            }))
        })?;
        let app_id = token_split.next().ok_or_else(|| {
            RbacError::System(fluent_message!("rbac-parse-res-str-fail",{
                "token":s
            }))
        })?;
        let app_id = app_id.parse::<u64>().map_err(|e| {
            RbacError::System(fluent_message!("rbac-parse-res-str-fail",{
                "token":s,
                "msg":e
            }))
        })?;
        let res_type = token_split
            .next()
            .ok_or_else(|| {
                RbacError::System(fluent_message!("rbac-parse-res-str-fail",{
                    "token":s
                }))
            })?
            .to_owned();
        let res_data = token_split
            .next()
            .ok_or_else(|| {
                RbacError::System(fluent_message!("rbac-parse-res-str-fail",{
                    "token":s,
                }))
            })?
            .to_string();
        Ok(ResCacheKey {
            user_id,
            app_id,
            res_type,
            res_data,
        })
    }
}

pub struct RbacResCache<'t> {
    pub res: &'t RbacRes,
}

impl RbacRes {
    pub fn cache(&self) -> RbacResCache<'_> {
        RbacResCache { res: self }
    }
}
