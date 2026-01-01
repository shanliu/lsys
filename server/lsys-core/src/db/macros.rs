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
/// 对指定结构体实现名为 $option_struct_name 的可选引用struct
/// @param $struct_name 结构体名
/// @param $option_struct_name 更改值临时存储的结构体名
/// @param {$name:$type} 字段名列表:类型列表
macro_rules! db_model_table_ref_define {
    ($self_var:ident,$struct_name:ident,$option_struct_name:ident,{$($name:ident[$column_name:literal]:$type:ty),+})=>{
        #[derive(PartialEq,Eq,Debug)]
        pub struct $option_struct_name<'t> {
            $(pub $name:Option<&'t $type>),*
        }
        impl<'t> $option_struct_name<'t> {
            #[allow(dead_code)]
            pub fn none_default()->Self{
                $option_struct_name {
                    $($name:None),*
                }
            }
        }
        impl<'t> $crate::db::InsertData<'t> for $option_struct_name<'t>
        {
            fn columns(&$self_var) -> Vec<$crate::db::FieldItem> {
                let mut vec = vec![];
                $(
                    if !$self_var.$name.is_none() {
                        vec.push($crate::db::FieldItem::new(stringify!($name),$column_name));
                    }
                ) *
                vec
            }
            fn sqlx_bind<'q>(&'q
                $self_var,
                field:&$crate::db::FieldItem,
                mut res: sqlx::query::Query<'q,sqlx::MySql,sqlx::mysql::MySqlArguments>,
            ) -> sqlx::query::Query<'q,sqlx::MySql,sqlx::mysql::MySqlArguments>{
                $crate::db_model_table_value_bind_define!(value_bind $self_var, res, field, {$($name),+});
            }
            fn sqlx_string(&$self_var,
                field:&$crate::db::FieldItem
            ) -> Option<String>{
                use $crate::db::SqlQuote;
                match field.name.as_str() {
                    $(
                        stringify!($name)=> {
                            Some($self_var.$name.map_or("".to_string(),|e|{
                                e.sql_quote().to_string()
                            }))
                        }
                    ) *
                    _=>None
                }
            }
        }
        impl<'t> $crate::db::ModelInsertData<'t,$option_struct_name<'t>> for $struct_name
        {
            fn insert_data(&'t $self_var) -> $option_struct_name<'t>{
                $option_struct_name {
                    $(
                       $name:Some(&$self_var.$name)
                    ),*
                }
            }
        }
        impl<'t> $crate::db::UpdateData<'t> for $option_struct_name<'t>
        {
            fn diff_columns(&$self_var) -> Vec<$crate::db::FieldItem> {
                let mut vec = vec![];
                $(
                    if !$self_var.$name.is_none() {
                        vec.push($crate::db::FieldItem::new(stringify!($name),$column_name));
                    }
                ) *
                vec
            }
            fn sqlx_bind<'q>(&'q
                $self_var,
                mut res: sqlx::query::Query<'q,sqlx::MySql,sqlx::mysql::MySqlArguments>,
            ) -> sqlx::query::Query<'q,sqlx::MySql,sqlx::mysql::MySqlArguments>
            {
                $(
                    if let Some(val) = $self_var.$name {
                        res = res.bind(val.clone());
                    }
                ) *
                res
            }
            fn sqlx_string(&$self_var,field:&$crate::db::FieldItem) -> Option<String>
            {
                use $crate::db::SqlQuote;
                match(field.name.as_str()){
                    $(
                        stringify!($name)=>{
                            if let Some(val) = $self_var.$name {
                                return Some(val.sql_quote().to_string())
                            }
                        }
                    ) *
                    _=>{}
                }
                None
            }
        }
        impl<'t> $crate::db::ModelUpdateData<'t, $option_struct_name<'t>> for $struct_name
        {
            fn diff(&'t $self_var, source_opt: &Option<&Self>) -> $option_struct_name<'t> {

                match source_opt {
                    Some(source) => {
                        $option_struct_name {$(
                            $name: if $self_var.$name != source.$name {
                                Some(&$self_var.$name)
                            } else {
                                None
                            }
                        ),*}
                    }
                    None => $option_struct_name {
                        $(
                           $name:Some(&$self_var.$name)
                        ),*
                    },
                }
            }
        }
    };
    ($struct_name:ident,$option_struct_name:ident,{$($name:ident[$column_name:literal]:$type:ty),+$(,)?})=>{
        $crate::db_model_table_ref_define!(self,$struct_name,$option_struct_name,{$($name[$column_name]:$type),+});
    };
}

#[macro_export]
/// 对指定结构体 ModelTableName ModelTableField
/// @param $struct_name 结构体名
/// @param $table_name 表名
/// @param {$name} 字段名列表
/// @param {$pk_name} 主键字段名列表
macro_rules! db_model_table_value_bind_define {
    (value_bind $self_var:ident,$res:expr,$val:expr,{$($name:ident),+})=>{
            match $val.name.as_str() {
                $(
                    stringify!($name)=> {
                        $res=$res.bind(&$self_var.$name);
                    }
                ) *
                _=>{}
            }
            return $res
    };
    ($self_var:ident,$struct_name:ident,$table_name:expr,{$($name:ident[$column_name:literal]),+})=>{
        impl $crate::db::ModelTableName for $struct_name {
            fn table_name() -> $crate::db::TableName {
                $crate::db::TableName::new($table_name)
            }
        }
        impl $crate::db::ModelTableField for $struct_name{
            fn table_column() -> $crate::db::TableFields {
                $crate::db::TableFields::new(vec![
                    $(
                        $crate::db::FieldItem::new(stringify!($name),$column_name)
                    ),*
                ])
            }
            fn query_sqlx_bind<'t>(
                &'t
                $self_var,
                field_val: &$crate::db::FieldItem,
                mut res: sqlx::query::Query<'t,sqlx::MySql,sqlx::mysql::MySqlArguments>,
            ) -> sqlx::query::Query<'t,sqlx::MySql,sqlx::mysql::MySqlArguments>
            {
                $crate::db_model_table_value_bind_define!(value_bind $self_var, res, field_val, {$($name),+});
            }
            fn query_as_sqlx_bind<'t, M>(
                &'t $self_var,
                field_val: &$crate::db::FieldItem,
                mut res:  sqlx::query::QueryAs<'t, sqlx::MySql, M, sqlx::mysql::MySqlArguments>,
            ) -> sqlx::query::QueryAs<'t, sqlx::MySql, M, sqlx::mysql::MySqlArguments>
            where
                for<'r> M: sqlx::FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin,
            {
                $crate::db_model_table_value_bind_define!(value_bind $self_var, res, field_val,{$($name),+});
            }
        }
    };
    ($struct_name:ident,$table_name:expr,{$($name:ident[$column_name:literal]),+})=>{
        $crate::db_model_table_value_bind_define!(self ,$struct_name,$table_name,{$($name[$column_name]),+});
    };
}

#[test]
fn test_model_define_bind_macro() {
    #[allow(unused)]
    pub struct UserModel {
        pub id: u32,
        pub nickname: String,
        pub gender: u8,
        pub headimg: Option<String>,
        pub password_id: u32,
    }
    crate::db_model_table_value_bind_define!(UserModel,"user",{
        id["id"],
        nickname["nickname"],
        gender["gender"],
        headimg["headimg"],
        password_id["password_id"]
    });
    crate::db_model_table_ref_define!(UserModel,UserModelRef,{
        id["id"]: u32,
        nickname["nickname"]: String,
        gender["gender"]: u8,
        headimg["headimg"]: Option<String>,
        password_id["password_id"]: u32,
    });
}

#[macro_export]
/// 对实现 none_default 方法的struct 用指定键值对快速创建结构 可由db_model_table_ref_define实现
/// @param $struct_name 结构体名
/// @param $key 字段名
/// @param $val 数据
macro_rules! model_option_set {
    ($struct_name:ident,{$($key:ident:$val:expr),*$(,)?})=>{
        {
            $struct_name{
                $(
                    $key:Some(&$val),
                )*
                ..$struct_name::none_default()
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
