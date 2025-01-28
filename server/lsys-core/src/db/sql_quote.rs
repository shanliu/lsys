/// 实现用于转义变量的trait
pub trait SqlQuote<OUT>
where
    OUT: std::fmt::Display,
{
    fn sql_quote(&self) -> OUT;
}

/// 保留原样的变量,不自动转义,当为自定义SQL 不能当字符串串处理时使用
pub struct SqlExpr<T: std::fmt::Display>(pub T);
impl<T: std::fmt::Display> SqlQuote<String> for SqlExpr<T> {
    fn sql_quote(&self) -> String {
        format!("{}", &self.0)
    }
}
macro_rules! array_join_to_str {
    ($self:expr) => {
        $self
            .into_iter()
            .map(|e| format!("{}", e.sql_quote()))
            .collect::<Vec<String>>()
            .join(",")
    };
}
macro_rules! option_to_str {
    ($self:expr) => {
        match $self {
            Some(str) => str.sql_quote().to_string(),
            None => "NULL".to_string(),
        }
    };
}
//常用类型
macro_rules! sql_quote_self {
    ($in_type:ty) => {
        impl SqlQuote<$in_type> for $in_type {
            fn sql_quote(&self) -> $in_type {
                *self
            }
        }
    };
}
sql_quote_self!(i8);
sql_quote_self!(i16);
sql_quote_self!(i32);
sql_quote_self!(i64);
sql_quote_self!(i128);
sql_quote_self!(u8);
sql_quote_self!(u16);
sql_quote_self!(u32);
sql_quote_self!(u64);
sql_quote_self!(u128);
sql_quote_self!(f32);
sql_quote_self!(f64);
sql_quote_self!(usize);
sql_quote_self!(isize);
impl SqlQuote<String> for char {
    fn sql_quote(&self) -> String {
        if (*self) == '\'' {
            "'\\''".to_string()
        } else {
            format!("'{self}'")
        }
    }
}
impl SqlQuote<u8> for bool {
    fn sql_quote(&self) -> u8 {
        (*self) as u8
    }
}
impl SqlQuote<String> for &str {
    fn sql_quote(&self) -> String {
        format!("'{}'", self.replace('\'', "\\'"))
    }
}
impl SqlQuote<String> for String {
    fn sql_quote(&self) -> String {
        format!("'{}'", self.replace('\'', "\\'"))
    }
}
//OPTION<常用类型>
macro_rules! sql_quote_option {
    ($in_type:ty) => {
        impl SqlQuote<String> for Option<$in_type> {
            fn sql_quote(&self) -> String {
                option_to_str!(self)
            }
        }
    };
}
sql_quote_option!(i8);
sql_quote_option!(i16);
sql_quote_option!(i32);
sql_quote_option!(i64);
sql_quote_option!(i128);
sql_quote_option!(u8);
sql_quote_option!(u16);
sql_quote_option!(u32);
sql_quote_option!(u64);
sql_quote_option!(u128);
sql_quote_option!(f32);
sql_quote_option!(f64);
sql_quote_option!(usize);
sql_quote_option!(isize);
sql_quote_option!(bool);
sql_quote_option!(char);
sql_quote_option!(String);
sql_quote_option!(&i8);
sql_quote_option!(&i16);
sql_quote_option!(&i32);
sql_quote_option!(&i64);
sql_quote_option!(&i128);
sql_quote_option!(&u8);
sql_quote_option!(&u16);
sql_quote_option!(&u32);
sql_quote_option!(&u64);
sql_quote_option!(&u128);
sql_quote_option!(&f32);
sql_quote_option!(&f64);
sql_quote_option!(&usize);
sql_quote_option!(&isize);
sql_quote_option!(&bool);
sql_quote_option!(&char);
sql_quote_option!(&String);
sql_quote_option!(&str);

//Vec<常用类型> or [常用类型]
macro_rules! sql_quote_array {
    ($in_type:ty) => {
        impl SqlQuote<String> for Vec<$in_type> {
            fn sql_quote(&self) -> String {
                array_join_to_str!(self)
            }
        }
        impl SqlQuote<String> for &Vec<$in_type> {
            fn sql_quote(&self) -> String {
                array_join_to_str!(self)
            }
        }
        impl SqlQuote<String> for [$in_type] {
            fn sql_quote(&self) -> String {
                array_join_to_str!(self)
            }
        }
        impl SqlQuote<String> for &[$in_type] {
            fn sql_quote(&self) -> String {
                array_join_to_str!(self)
            }
        }
    };
}
sql_quote_array!(i8);
sql_quote_array!(i16);
sql_quote_array!(i32);
sql_quote_array!(i64);
sql_quote_array!(i128);
sql_quote_array!(u8);
sql_quote_array!(u16);
sql_quote_array!(u32);
sql_quote_array!(u64);
sql_quote_array!(u128);
sql_quote_array!(f32);
sql_quote_array!(f64);
sql_quote_array!(usize);
sql_quote_array!(isize);
sql_quote_array!(bool);
sql_quote_array!(&str);
sql_quote_array!(String);

#[macro_export]
/// 转义SQL生成,对字符串中的单引号加反斜杠
macro_rules! sql_format {
    ($fmt:expr) => {
        format!($fmt)
    };
    ($fmt:expr,$($argsname:tt=$argsval:expr),+$(,)?) => {
        format!($fmt,$($argsname=$argsval.sql_quote()) ,+)
    };
    ($fmt:expr,$($args:expr),+$(,)?) => {
        format!($fmt,$($args.sql_quote()),+)
    };
}

#[test]
fn test_sql_format_macro() {
    assert_eq!(sql_format!("{var_i32}", var_i32 = 1), "1");
    assert_eq!(sql_format!("{}", 1_i8), "1");

    let aa = || 1;
    assert_eq!(sql_format!("{}", aa()), "1");
    fn bb() -> i8 {
        1
    }
    assert_eq!(sql_format!("{}", bb()), "1");

    assert_eq!(sql_format!("{}", "1'1'1"), "'1\\'1\\'1'");
    assert_eq!(sql_format!("{}", "1'1'1".to_string()), "'1\\'1\\'1'");

    assert_eq!(sql_format!("{}", '\''), "'\\''");

    assert_eq!(
        sql_format!("{}", vec!["1", "2'2'2'2'"]),
        "'1','2\\'2\\'2\\'2\\''"
    );

    assert_eq!(
        sql_format!("{}", ["1", "2'2'2'2'"]),
        "'1','2\\'2\\'2\\'2\\''"
    );

    assert_eq!(
        sql_format!("{}", ["1".to_string(), "2'2'2'2'".to_string()]),
        "'1','2\\'2\\'2\\'2\\''"
    );

    assert_eq!(
        sql_format!("{}", vec!["1".to_string(), "2'2'2'2'".to_string()]),
        "'1','2\\'2\\'2\\'2\\''"
    );

    assert_eq!(sql_format!("{}", SqlExpr("select 1 as a")), "select 1 as a");
}
