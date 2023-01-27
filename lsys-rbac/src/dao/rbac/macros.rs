// 给指定结构实现验证码功能
macro_rules! check_length {
    ($fluent:expr,$var:ident,$name:literal,$max:literal) => {{
        let $var = $var.trim().to_string();
        if $var.is_empty() || $var.len() > $max {
            return Err(UserRbacError::System(get_message!(
                $fluent,
                concat!("res-", $name, "-wrong"),
                concat!($name, " length need 1-", $max, " char")
            )));
        }
        $var
    }};
}
