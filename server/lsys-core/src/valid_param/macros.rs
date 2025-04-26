#[macro_export]
macro_rules! valid_param {
    //校验单个变量
    ($db_field:expr,[$($valid_rule:ident),*]) => {{
        //这里实现校验变量
    }};
    //校验一批变量
    ($({$db_field:expr,[$($valid_rule:ident),*]},*)+) => {{
        //这里实现校验变量
    }};
}
