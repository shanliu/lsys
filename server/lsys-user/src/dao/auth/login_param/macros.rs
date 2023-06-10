macro_rules! auth_user_not_found_map {
    ($fluent:expr,$name:expr,$login_type:literal)=>{
        |err:UserAccountError|{
            return match err {
                UserAccountError::Sqlx(sqlx::Error::RowNotFound) => {
                    UserAuthError::UserNotFind(
                        get_message!($fluent,"auth-not-user",concat!($login_type," login, account {$user} not find",),["user"=>$name])
                    )
                }
                _ => {
                    UserAuthError::UserAccount(err)
                }
            }
        }
    };
}

// 给指定结构实现验证码功能
macro_rules! impl_auth_valid_code_method {
    ($valid_type:literal,{$($name:ident:$name_type:ty),+$(,)*},$key_block:block,$save_time:expr) => {
        /// 验证码生成
        pub fn valid_code(redis: deadpool_redis::Pool) -> lsys_core::ValidCode {
            lsys_core::ValidCode::new(redis, $valid_type.to_string(),true)
        }
        /// 获取验证码
        pub async fn valid_code_set<T: lsys_core::ValidCodeData>(
            redis: deadpool_redis::Pool,
            valid_code_data:&mut T,
            $($name:$name_type),+
        ) -> lsys_core::ValidCodeResult<(String, usize)> {
            let key = $key_block;
            let valid_code=Self::valid_code(redis);
            let res = valid_code.delay_code(&key,valid_code_data).await;
            match res {
                Ok(out) => {
                    Ok(out)
                }
                Err(lsys_core::ValidCodeError::DelayTimeout(_))=>{
                    let code = valid_code.set_code(&key,valid_code_data).await?;
                    Ok(code)
                }
                Err(err) => Err(err),
            }
        }
        /// 验证码构造器
        pub fn valid_code_builder(
        ) -> lsys_core::ValidCodeDataRandom{
            lsys_core::ValidCodeDataRandom::new(lsys_core::ValidCodeTime::time($save_time))
        }
        /// 检测验证码
        pub async fn valid_code_check(
            redis:deadpool_redis::Pool,
            code: &String,
            $($name:$name_type),+
        ) -> UserAuthResult<()> {
            let key = $key_block;
            Self::valid_code(redis).check_code(&key, code).await?;
            Ok(())
        }
        pub async fn valid_code_clear(
            redis: deadpool_redis::Pool,
            $($name:$name_type),+
        ) -> UserAuthResult<()> {
            let key = $key_block;
            let mut builder=Self::valid_code_builder();
            Self::valid_code(redis).clear_code(&key,&mut builder ).await?;
            Ok(())
        }
    };
}
