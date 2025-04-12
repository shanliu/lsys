macro_rules! auth_user_not_found_map {
    ($name:expr,$login_type:literal)=>{
        |err:AccountError|{
            return match err {
                AccountError::Sqlx(sqlx::Error::RowNotFound) => {
                    AccountError::UserNotFind(
                        lsys_core::fluent_message!("auth-not-user",{"name":$name})//concat!($login_type," login, account {$user} not find",)
                    )
                }
                _ => {
                    err
                }
            }
        }
    };
}
