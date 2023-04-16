// 提供一个方式 用于统一定义验证所需资源跟验证方式
// !!!非必须,可外部自行组织!!!
use super::{Rbac, RbacAccess, RoleRelationKey, UserRbacResult};
use serde::{Deserialize, Serialize};

// 授权依赖类型
pub type RbacCheckDepend = dyn RbacCheck + std::marker::Sync + std::marker::Send;
// 授权检测trait,使用时统一定义授权
#[async_trait::async_trait]
pub trait RbacCheck {
    //当前授权依赖授权列表
    fn depends(&self) -> Vec<Box<RbacCheckDepend>> {
        vec![]
    }
    // 进行授权当前
    async fn check<'t>(
        &self,
        access: &'t RbacAccess,
        relation: &'t [RoleRelationKey],
    ) -> UserRbacResult<()>;
}

impl Rbac {
    // 检查实现了RbacCheck trait的授权检查
    #[async_recursion::async_recursion]
    pub async fn check<'t>(
        &self,
        //待检测的权限结构
        //把多个资源跟相关依赖关系一个结构中
        //外部调用检测权限时传入该封装结构
        check: &'t RbacCheckDepend,
        //用户间关系KEY列表
        relation: Option<&'t [RoleRelationKey]>,
    ) -> UserRbacResult<()> {
        for pr in check.depends() {
            self.check(pr.as_ref(), relation).await?
        }
        check
            .check(&self.access, relation.unwrap_or_default())
            .await
    }
}

// 提供一个方式方便收集代码中使用到的关系数据
// 关系模板获取trait 定义,用在 access_relation_tpl 宏中
// 资源模板
#[derive(Debug, Serialize, Deserialize)]
pub struct RelationTpl {
    pub key: &'static str, //资源KEY
    pub user: bool,        //系统资源还是用户资源
}
pub trait RbacRelationTpl {
    fn extend(&self, relation: &[RoleRelationKey]) -> Vec<RoleRelationKey> {
        let mut e = self.relation_data();
        e.extend(relation.to_owned());
        e
    }
    fn relation_data(&self) -> Vec<RoleRelationKey>;
    fn tpl_data() -> Vec<RelationTpl>;
}

#[macro_export]
macro_rules! access_relation_tpl {
    ($($res_type:ty),+) => {{
        use $crate::dao::RbacRelationTpl;
        use $crate::dao::RelationTpl;
        let mut data = Vec::<RelationTpl>::new();
        $(
            data.extend(<$res_type>::tpl_data());

        )+
        data
    }};
}

// 提供一个方式方便收集代码中使用到的资源
// !!!非必须,可外部自行组织!!!
// 通过trait加宏操作方式实现 一般在实现了 RbacCheck trait 的结构上实现

// 资源模板
#[derive(Debug, Serialize, Deserialize)]
pub struct ResTpl {
    pub tags: Vec<&'static str>, //资源建议的TAG,分类用
    pub key: &'static str,       //资源KEY
    pub user: bool,              //系统资源还是用户资源
    pub ops: Vec<&'static str>,  //资源包含操作
}

// 资源模板获取trait 定义,用在 access_res_tpl 宏中
pub trait RbacResTpl {
    fn tpl_data() -> Vec<ResTpl>; //返回授权操作包含的资源列表
}

#[macro_export]
macro_rules! access_res_tpl {
    ($($res_type:ty),+) => {{
        use $crate::dao::RbacResTpl;
        let mut data = Vec::<$crate::dao::ResTpl>::new();
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
                        for ot in e.tags.iter() {
                            if !tmp.tags.contains(ot) {
                                tmp.tags.push(*ot)
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
    impl RbacResTpl for R1 {
        fn tpl_data() -> Vec<ResTpl> {
            vec![ResTpl {
                tags: vec![],
                user: false,
                key: "dd${aaa}",
                ops: vec!["ddd", "bbb"],
            }]
        }
    }
    struct R2 {}
    impl RbacResTpl for R2 {
        fn tpl_data() -> Vec<ResTpl> {
            vec![
                ResTpl {
                    tags: vec![],
                    user: false,
                    key: "dd${aaa}",
                    ops: vec!["ccc", "ddd"],
                },
                ResTpl {
                    tags: vec![],
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
