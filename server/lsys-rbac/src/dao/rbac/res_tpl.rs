// 提供一个方式方便收集代码中使用到的资源
// !!!非必须,可外部自行组织!!!
// 通过trait加宏操作方式实现 一般在实现了 RbacCheck trait 的结构上实现
// 关于如何通过 RbacCheck trait 定义权限验证方式 可参考: ./check.rs

use serde::{Deserialize, Serialize};

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

//////////////////// 测试用例 ////////////////////
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
