use std::fmt::{Display, Formatter};

use lazy_static::lazy_static;
use lsys_rbac::dao::AccessRes;
use serde::Serialize;

#[derive(Clone)]
pub struct ScopeItem {
    pub name: &'static str,
    pub res: Vec<lsys_rbac::dao::AccessRes>,
}

#[derive(Serialize)]
pub struct ShowScopeItem {
    pub name: &'static str,
}

#[derive(Debug)]
pub enum ScopeError {
    Parse(String),
}
impl Display for ScopeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

lazy_static! {
    static ref SCOPE_GROUP: Vec<ScopeItem> = vec![
        ScopeItem {
            name: "user_info",
            res: access_res!(["app", access_op!(["res-view-all", true])]),
        },
        ScopeItem {
            name: "user_email",
            res: access_res!(["app", access_op!(["res-view-all", true])]),
        },
        ScopeItem {
            name: "user_mobile",
            res: access_res!(["app", access_op!(["res-view-all", true])]),
        },
    ];
}

pub struct AppScope<'t> {
    pub inner: Vec<&'t ScopeItem>,
}
impl<'t> AppScope<'t> {
    pub fn all_scope() -> &'t Vec<ScopeItem> {
        &SCOPE_GROUP
    }
    pub fn to_check_res(&self) -> Vec<Vec<AccessRes>> {
        let mut out = vec![];
        for item in self.inner.clone() {
            out.extend(item.res.iter().map(|e| e.to_owned()));
        }
        vec![out]
    }
    pub fn to_show_data(&self) -> Vec<ShowScopeItem> {
        todo!()
    }
}

impl<'t> TryFrom<&str> for AppScope<'t> {
    type Error = ScopeError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let spoces = value.split(',').collect::<Vec<&str>>();
        let find_scope = AppScope::all_scope()
            .iter()
            .filter(|e| spoces.contains(&e.name))
            .collect::<Vec<&ScopeItem>>();
        Ok(AppScope { inner: find_scope })
    }
}
