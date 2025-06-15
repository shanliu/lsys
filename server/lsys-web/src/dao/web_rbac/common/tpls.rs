// 提供一个方便方式 来收集代码中使用到的资源
// !!!非必须,可外部自行组织!!!
// 通过trait加宏操作方式实现 一般在实现了 RbacCheck trait 的结构上实现

// 资源模板
#[derive(Debug)]
pub struct CheckResTpl {
    pub key: &'static str,      //资源KEY
    pub data: bool,             //静态资源还是动态资源
    pub user: bool,             //系统资源还是用户资源
    pub ops: Vec<&'static str>, //资源包含操作
}

// 资源模板获取trait 定义,用在 access_res_tpl 宏中
pub trait RbacCheckResTpl {
    fn tpl_data() -> Vec<CheckResTpl>; //返回授权操作包含的资源列表
}

macro_rules! access_res_tpl {
    ($($res_type:ty),+$(,)*) => {{
        use $crate::dao::RbacCheckResTpl;
        let mut data = Vec::<$crate::dao::CheckResTpl>::new();
        $(
            let tdat = <$res_type>::tpl_data();
            for e in tdat.iter() {
                for tmp in data.iter_mut() {
                    if tmp.key == e.key && tmp.user == e.user && tmp.data == e.data{
                        for ot in e.ops.iter() {
                            if !tmp.ops.contains(ot) {
                                tmp.ops.push(*ot)
                            }
                        }
                    }
                }
            }
            for tmp in tdat {
                if !data.iter().any(|e| tmp.key == e.key && tmp.user == e.user && tmp.data == e.data) {
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
                data: false,
                key: "dd",
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
                    data: false,
                    key: "dd",
                    ops: vec!["ccc", "ddd"],
                },
                CheckResTpl {
                    user: false,
                    data: false,
                    key: "oooo",
                    ops: vec!["ccc", "ddd"],
                },
            ]
        }
    }
    //data
    let res = access_res_tpl!(R2, R1);
    assert_eq!(res.first().unwrap().key, "dd");
    assert_eq!(res.first().unwrap().ops.len(), 3);
    assert_eq!(res.len(), 2);
}
