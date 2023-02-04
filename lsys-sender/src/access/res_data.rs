use std::vec;

use lsys_web::JsonResult;

macro_rules! res_data {
    ($res:ident) => {
        crate::dao::access::ResData::$res.to_access_res()?
    };
    ($res:ident($($var:expr),+$(,)*)) => {
        crate::access::ResData::$res($($var),+).to_access_res()?
    };
}

pub struct RoleOpCheck {
    pub op_id: u64,
    pub op_user_id: u64,
}

pub enum ResData {
    AppDecode(u64),
}

impl ResData {
    /// 转换为待检测资源
    #[allow(clippy::type_complexity)]
    pub fn to_access_data(
        &self,
    ) -> JsonResult<(Vec<Vec<lsys_rbac::dao::AccessRes>>, Option<Vec<Self>>)> {
        match self {
            ResData::AppDecode(app_id) => Ok((
                vec![access_res!([
                    format!("app-{}", app_id),
                    access_op!(["global-app-decode", true])
                ])],
                None,
            )),
        }
    }
}
impl ResData {
    pub fn to_access_res(&self) -> JsonResult<Vec<Vec<lsys_rbac::dao::AccessRes>>> {
        let (mut res, dependency) = self.to_access_data()?;
        if let Some(dep) = dependency {
            for tmp in dep {
                res = lsys_rbac::dao::RbacAccess::merge_access_res(&tmp.to_access_res()?, &res);
            }
        }
        Ok(res)
    }
}
