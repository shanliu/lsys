//字符串转数字宏定义
macro_rules! deserialize_define_number_fn {
    ( $value:expr,$as_method:ident,$parse_type:ty) => {
        match $value {
            serde_json::Value::Number(n) => {
                if let Some(i) = n.$as_method() {
                    let out = <$parse_type>::try_from(i).map_err(serde::de::Error::custom)?;
                    out
                } else {
                    return Err(serde::de::Error::custom("invalid number"));
                }
            }
            serde_json::Value::String(s) => {
                s.parse::<$parse_type>().map_err(serde::de::Error::custom)?
            }
            _ => return Err(serde::de::Error::custom("expected number, string")),
        }
    };
    ($fn_name:ident,$vec_fn_name:ident,$fn_option_name:ident,$vec_fn_option_name:ident,$as_method:ident,$parse_type:ty) => {
        pub fn $fn_name<'de, D>(deserializer: D) -> Result<$parse_type, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::Deserialize;
            Ok(deserialize_define_number_fn!(
                serde_json::Value::deserialize(deserializer)?,
                $as_method,
                $parse_type
            ))
        }
        pub fn $vec_fn_name<'de, D>(deserializer: D) -> Result<Vec<$parse_type>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::Deserialize;
            let values: Vec<serde_json::Value> = Deserialize::deserialize(deserializer)?;
            let mut out = Vec::with_capacity(values.len());
            for val in values {
                out.push(deserialize_define_number_fn!(val, $as_method, $parse_type));
            }
            Ok(out)
        }
        pub fn $fn_option_name<'de, D>(deserializer: D) -> Result<Option<$parse_type>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::Deserialize;
            let value = Option::<serde_json::Value>::deserialize(deserializer)?;
            let out = match value {
                Some(val) => Ok(deserialize_define_number_fn!(val, $as_method, $parse_type)),
                None => return Ok(None),
            };
            out.map(Some)
        }
        pub fn $vec_fn_option_name<'de, D>(
            deserializer: D,
        ) -> Result<Option<Vec<$parse_type>>, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use serde::Deserialize;
            let value = Option::<Vec<serde_json::Value>>::deserialize(deserializer)?;
            let out = match value {
                Some(values) => {
                    let mut out = Vec::with_capacity(values.len());
                    for val in values {
                        out.push(deserialize_define_number_fn!(val, $as_method, $parse_type));
                    }
                    Ok(out)
                }
                None => return Ok(None),
            };
            out.map(Some)
        }
    };
}

//具体类型函数实现
deserialize_define_number_fn!(
    deserialize_i8,
    deserialize_vec_i8,
    deserialize_option_i8,
    deserialize_option_vec_i8,
    as_i64,
    i8
);
deserialize_define_number_fn!(
    deserialize_i16,
    deserialize_vec_i16,
    deserialize_option_i16,
    deserialize_option_vec_i16,
    as_i64,
    i16
);
deserialize_define_number_fn!(
    deserialize_i32,
    deserialize_vec_i32,
    deserialize_option_i32,
    deserialize_option_vec_i32,
    as_i64,
    i32
);
deserialize_define_number_fn!(
    deserialize_i64,
    deserialize_vec_i64,
    deserialize_option_i64,
    deserialize_option_vec_i64,
    as_i64,
    i64
);
deserialize_define_number_fn!(
    deserialize_i128,
    deserialize_vec_i128,
    deserialize_option_i128,
    deserialize_option_vec_i128,
    as_i64,
    i128
);
deserialize_define_number_fn!(
    deserialize_u8,
    deserialize_vec_u8,
    deserialize_option_u8,
    deserialize_option_vec_u8,
    as_u64,
    u8
);
deserialize_define_number_fn!(
    deserialize_u16,
    deserialize_vec_u16,
    deserialize_option_u16,
    deserialize_option_vec_u16,
    as_u64,
    u16
);
deserialize_define_number_fn!(
    deserialize_u32,
    deserialize_vec_u32,
    deserialize_option_u32,
    deserialize_option_vec_u32,
    as_u64,
    u32
);
deserialize_define_number_fn!(
    deserialize_u64,
    deserialize_vec_u64,
    deserialize_option_u64,
    deserialize_option_vec_u64,
    as_u64,
    u64
);
deserialize_define_number_fn!(
    deserialize_u128,
    deserialize_vec_u128,
    deserialize_option_u128,
    deserialize_option_vec_u128,
    as_u64,
    u128
);
deserialize_define_number_fn!(
    deserialize_f64,
    deserialize_vec_f64,
    deserialize_option_f64,
    deserialize_option_vec_f64,
    as_f64,
    f64
);

macro_rules! deserialize_define_string_fn {
    ( $value:expr) => {
        match $value {
            serde_json::Value::String(n) => Ok(n),
            serde_json::Value::Number(n) => Ok(n.to_string()),
            serde_json::Value::Bool(s) => {
                if s {
                    Ok("1".to_string())
                } else {
                    Ok("0".to_string())
                }
            }
            _ => Err(serde::de::Error::custom("expected number, string")),
        }
    };
}
//数字转字符串
pub fn deserialize_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    deserialize_define_string_fn!(serde_json::Value::deserialize(deserializer)?)
}
pub fn deserialize_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let out = match value {
        Some(val) => deserialize_define_string_fn!(val),
        None => return Ok(None),
    };
    out.map(Some)
}

//字符串转布尔宏定义
macro_rules! deserialize_define_bool_fn {
    ( $value:expr) => {
        match $value {
            serde_json::Value::String(n) => {
                if n == *"1" || n == *"true" || n == *"True" || n == *"TRUE" {
                    Ok(true)
                } else if n == *"0" || n == *"false" || n == *"False" || n == *"FALSE" {
                    Ok(false)
                } else {
                    Err(serde::de::Error::custom("invalid bool"))
                }
            }
            serde_json::Value::Number(n) => Ok(n
                .as_u64()
                .ok_or_else(|| serde::de::Error::custom("invalid bool"))?
                == 0),
            serde_json::Value::Bool(s) => Ok(s),
            _ => Err(serde::de::Error::custom("invalid bool")),
        }
    };
}

pub fn deserialize_bool<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    deserialize_define_bool_fn!(serde_json::Value::deserialize(deserializer)?)
}
pub fn deserialize_option_bool<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::Deserialize;
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;
    let out = match value {
        Some(val) => deserialize_define_bool_fn!(val),
        None => return Ok(None),
    };
    out.map(Some)
}
