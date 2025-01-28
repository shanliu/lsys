
//RBAC中资源相关实现

use lsys_core::fluent_message;

use std::str::FromStr;

use std::vec;

use crate::dao::result::RbacError;

use super::RbacOp;



#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct OpCacheKey {
    pub op_key: String, //资源类型
    pub user_id: u64,    //资源用户ID
}

impl std::fmt::Display for OpCacheKey {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}-{}", self.user_id, self.op_key)
    }
}

impl FromStr for OpCacheKey {
    type Err = RbacError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut token_split = s.split('-');
        let user_id = token_split.next().ok_or_else(|| {
            RbacError::System(fluent_message!("parse-op-str-fail",{
                "token":s
            }))
        })?;
        let user_id = user_id.parse::<u64>().map_err(|e| {
            RbacError::System(fluent_message!("parse-op-str-fail",{
                "token":s,
                "msg":e
            }))
        })?;
        let op_key = token_split.next().ok_or_else(|| {
            RbacError::System(fluent_message!("parse-op-str-fail",{
                "token":s
            }))
        })?.to_owned();
        Ok(Self { user_id,op_key })
    }
}


pub struct RbacOpCache<'t> {
    pub op: &'t RbacOp,
}

impl RbacOp {
    pub fn cache(&self) -> RbacOpCache<'_> {
        RbacOpCache { op: self }
    }
}

