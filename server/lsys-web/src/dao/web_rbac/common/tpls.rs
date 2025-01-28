use std::convert::From;

use lsys_rbac::dao::AccessSessionRole;
// 提供一个方便方式 收集代码中使用到的关系数据
// !!!非必须,可外部自行组织!!!
// 关系模板获取trait 定义,用在 access_relation_tpl 宏中
// 资源模板
#[derive(Debug)]
pub struct CheckRelationTpl {
    pub key: &'static str, //资源KEY
    pub user: bool,        //系统资源还是用户资源
}
#[derive(Clone)]
pub struct CheckRelationRole {
    pub role_key: String,
    pub user_id: u64,
}

impl<'t> From<&'t CheckRelationRole> for AccessSessionRole<'t> {
    fn from(value: &'t CheckRelationRole) -> Self {
        AccessSessionRole {
            role_key: &value.role_key,
            user_id: value.user_id,
        }
    }
}

#[derive(Clone, Default)]
pub struct CheckRelationData {
    role_data: Vec<CheckRelationRole>,
}

impl CheckRelationData {
    pub fn to_session_role(&self) -> Vec<AccessSessionRole<'_>> {
        self.role_data
            .iter()
            .map(|e| AccessSessionRole {
                role_key: &e.role_key,
                user_id: e.user_id,
            })
            .collect::<Vec<_>>()
    }
    pub fn extend(&mut self, relation: &CheckRelationData) -> &mut Self {
        self.role_data.extend(
            relation
                .role_data
                .iter()
                .map(|e| e.to_owned())
                .collect::<Vec<_>>(),
        );
        self
    }
}

impl From<Vec<CheckRelationRole>> for CheckRelationData {
    fn from(role_data: Vec<CheckRelationRole>) -> Self {
        CheckRelationData { role_data }
    }
}

pub trait RbacCheckRelationTpl {
    fn relation_data(&self) -> CheckRelationData;
    fn tpl_data() -> Vec<CheckRelationTpl>;
}

#[macro_export]
macro_rules! access_relation_tpl {
    ($($res_type:ty),+) => {{
        use $crate::dao::RbacCheckRelationTpl;
        use $crate::dao::CheckRelationTpl;
        let mut data = Vec::<CheckRelationTpl>::new();
        $(
            data.extend(<$res_type>::tpl_data());
        )+
        data
    }};
}

// 提供一个方便方式 来收集代码中使用到的资源
// !!!非必须,可外部自行组织!!!
// 通过trait加宏操作方式实现 一般在实现了 RbacCheck trait 的结构上实现

// 资源模板
#[derive(Debug)]
pub struct CheckResTpl {
    pub key: &'static str,      //资源KEY
    pub user: bool,             //系统资源还是用户资源
    pub ops: Vec<&'static str>, //资源包含操作
}

// 资源模板获取trait 定义,用在 access_res_tpl 宏中
pub trait RbacCheckResTpl {
    fn tpl_data() -> Vec<CheckResTpl>; //返回授权操作包含的资源列表
}

#[macro_export]
macro_rules! access_res_tpl {
    ($($res_type:ty),+) => {{
        use $crate::dao::RbacCheckResTpl;
        let mut data = Vec::<$crate::dao::CheckResTpl>::new();
        $(
            let tdat = <$res_type>::tpl_data();
            for e in tdat.iter() {
                for tmp in data.iter_mut() {
                    if tmp.key == e.key {
                        for ot in e.ops.iter() {
                            if !tmp.ops.contains(ot) {
                                tmp.ops.push(*ot)
                            }
                        }
                    }
                }
            }
            for tmp in tdat {
                if !data.iter().any(|e| e.key == tmp.key) {
                    data.push(tmp)
                }
            }
        )+
        data
    }};
}

//////////////////// res tpl 测试用例 ////////////////////
#[test]
fn test_tpl() {
    struct R1 {}
    impl RbacCheckResTpl for R1 {
        fn tpl_data() -> Vec<CheckResTpl> {
            vec![CheckResTpl {
                user: false,
                key: "dd${aaa}",
                ops: vec!["ddd", "bbb"],
            }]
        }
    }
    struct R2 {}
    impl RbacCheckResTpl for R2 {
        fn tpl_data() -> Vec<CheckResTpl> {
            vec![
                CheckResTpl {
                    user: false,
                    key: "dd${aaa}",
                    ops: vec!["ccc", "ddd"],
                },
                CheckResTpl {
                    user: false,
                    key: "oooo",
                    ops: vec!["ccc", "ddd"],
                },
            ]
        }
    }
    //data
    let res = access_res_tpl!(R2, R1);
    assert_eq!(res.first().unwrap().key, "dd${aaa}");
    assert_eq!(res.first().unwrap().ops.len(), 3);
    assert_eq!(res.len(), 2);
}
