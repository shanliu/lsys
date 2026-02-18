use sqlx::Pool;
use sqlx::Transaction;

pub trait DBOptionExecutorTransaction {
    fn as_executor(&mut self) -> &mut sqlx::mysql::MySqlConnection;
}

impl DBOptionExecutorTransaction for Transaction<'_, sqlx::MySql> {
    fn as_executor(&mut self) -> &mut sqlx::mysql::MySqlConnection {
        #[allow(clippy::explicit_auto_deref)]
        &mut **self
    }
}

pub trait DBOptionExecutorPool {
    fn as_executor(&self) -> &Self;
}

impl DBOptionExecutorPool for Pool<sqlx::MySql> {
    fn as_executor(&self) -> &Self {
        self
    }
}

#[macro_export]
/// 对包含块代码中的链接变量选择事物或连接池
/// @param $execute_name  $block块中用到的连接变量名
/// @param $block 执行代码,块中使用 $execute_name.as_executor() 获取连接
/// @param $transaction_executor Option 当存在时$block中 $execute_name 变量用此值
/// @param $poll_executor 事务$transaction_executor为NONE时 $execute_name 变量用此值
macro_rules! db_option_executor {
    ($execute_name:tt,$block:block,$transaction_executor:expr,$poll_executor:expr) => {
        match $transaction_executor {
            Some($execute_name) => {
                #[allow(unused_imports)]
                use $crate::db::DBOptionExecutorTransaction;
                $block
            }
            None => {
                #[allow(unused_imports)]
                use $crate::db::DBOptionExecutorPool;
                let $execute_name = $poll_executor;
                $block
            }
        }
    };
}

#[macro_export]
/// 对状态类型的结构提供辅助方法
/// @param $enum_name 状态枚举
/// @param $type 状态的类型
/// @param $item 可选值列表
macro_rules! db_model_enum_status_define {
    ($self_var:ident,$enum_name:ident,$type:ty,{$($item:expr),*$(,)?})=>{
        #[allow(dead_code)]
        impl $enum_name{
            pub fn eq(&self,eq:$type)->bool{
                return (*self as $type)==eq;
            }
            pub fn to(self)->$type{
                return self as $type
            }
            pub fn fluent(&self)->$crate::FluentMessage{
                 $(
                    if *self ==$item {
                       return  $crate::FluentMessage {
                            id: format!("status-{}",stringify!($item).replace(' ',"").replace("::","-")),
                            crate_name: env!("CARGO_PKG_NAME").to_string(),
                            data: vec![],
                        }
                    }
                )*
                $crate::FluentMessage {
                    id: format!("status-{}-{}",stringify!($enum_name),(*self as $type)),
                    crate_name: env!("CARGO_PKG_NAME").to_string(),
                    data: vec![],
                }
            }
        }
		impl $crate::db::SqlQuote<$type> for $enum_name {
			fn sql_quote(&self) -> $type {
				*self as $type
			}
		}
        impl std::convert::TryFrom<$type> for $enum_name {
            type Error=sqlx::Error;
            fn try_from(value:  $type) -> Result<Self, Self::Error> {
                $(
                    if ($item as $type) ==value {
                        return Ok($item);
                    }
                )*
                return Err(sqlx::Error::TypeNotFound { type_name: format!("{}({}):[{}] ",stringify!($enum_name),stringify!($type),value) })
            }
        }
    };
    ($enum_name:ident,$type:ty,{$($item:expr),*$(,)?})=>{
        $crate::db_model_enum_status_define!(self ,$enum_name,$type,{$(
            $item,
        )*});
    };
    ($enum_name:ident,$type:ty)=>{
        $crate::db_model_enum_status_define!(self ,$enum_name,$type,{});
    };
}

#[test]
fn test_model_enum_status() {
    #[derive(PartialEq, Eq, Clone, Copy)]
    enum UserModelStatus {
        Statu1 = 1,
        Statu2 = 2,
    }
    crate::db_model_enum_status_define!(UserModelStatus,u8,{
        UserModelStatus::Statu1,
        UserModelStatus::Statu2
    });
    assert!(UserModelStatus::Statu1.eq(1));
    assert!(!UserModelStatus::Statu1.eq(2));
    assert!(UserModelStatus::Statu2.eq(2));
    let status: UserModelStatus = 2.try_into().unwrap();
    assert!(status == UserModelStatus::Statu2);
    let status: Result<UserModelStatus, _> = 3.try_into();
    assert!(status.is_err());
}
