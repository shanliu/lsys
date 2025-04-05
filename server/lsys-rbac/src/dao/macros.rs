// 给指定结构实现验证码功能
macro_rules! check_length {
    ($var:ident,$name:literal,$max:literal) => {{
        let $var = $var.trim().to_string();
        if $var.is_empty() || $var.len() > $max {
            return Err(crate::dao::result::RbacError::System(lsys_core::fluent_message!("rbac-check-length",
                {
                    "key":$name,
                    "msg": concat!(" length need 1-", $max, " char")
                }
            )));
        }
        $var
    }};
}
