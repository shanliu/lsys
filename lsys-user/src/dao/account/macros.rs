// 给指定结构实现验证码功能
macro_rules! impl_account_valid_code_method {
    ($valid_type:literal,{$($name:ident:$name_type:ty),+$(,)*},$key_block:block,$save_time:expr) => {
         impl_account_valid_code_method!(self,$valid_type,{$($name:$name_type),*},$key_block,$save_time);
    };
    ($me:ident,$valid_type:literal,{$($name:ident:$name_type:ty),+$(,)*},$key_block:block,$save_time:expr) => {
        /// 验证码生成
        pub fn valid_code(&$me) -> lsys_core::ValidCode {
            lsys_core::ValidCode::new($me.redis.clone(), $valid_type.to_string(),true)
        }
        /// 获取验证码
        pub async fn valid_code_set<T: lsys_core::ValidCodeData>(
            &$me,
            valid_code_data:&mut T,
            $($name:$name_type),+
        ) -> lsys_core::ValidCodeResult<(String, usize)> {
            let key = $key_block;
            let res=$me.valid_code().delay_code(&key,valid_code_data).await;
            match res {
                Ok(out) => {
                    Ok(out)
                }
                Err(lsys_core::ValidCodeError::DelayTimeout(_))=>{
                   let out = $me.valid_code().set_code(&key,valid_code_data).await?;
                   Ok(out)
                }
                Err(err) => Err(err),
            }
        }
        /// 验证码构造器
        pub fn valid_code_builder(
            &$me
        ) -> lsys_core::ValidCodeDataRandom{
            lsys_core::ValidCodeDataRandom::new(lsys_core::ValidCodeTime::time($save_time))
        }
        /// 检测验证码
        pub async fn valid_code_check(
            &$me,
            code: &String,
            $($name:$name_type),+
        ) -> UserAccountResult<()> {
            let key = $key_block;
            $me.valid_code().check_code(&key, code).await?;
            Ok(())
        }
        pub async fn valid_code_clear(
            &$me,
            $($name:$name_type),+
        ) -> UserAccountResult<()> {
            let key = $key_block;
            let mut builder=$me.valid_code_builder();
            $me.valid_code().clear_code(&key,&mut builder ).await?;
            Ok(())
        }
    };
}

#[test]
fn valid_code_test() {
    use crate::dao::account::UserAccountResult;
    use redis::aio::ConnectionManager;
    use std::sync::Arc;
    use tokio::sync::Mutex;
    #[allow(dead_code)]
    struct Test1 {
        redis: Arc<Mutex<ConnectionManager>>,
    }
    #[allow(dead_code)]
    impl Test1 {
        impl_account_valid_code_method!("sss",{
            area_code:&String,
            mobile:&str,
        },{area_code.to_owned()+mobile},10);
    }
}
