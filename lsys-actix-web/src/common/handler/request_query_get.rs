use actix_web::error::ErrorBadRequest;
use actix_web::web::Query;
use actix_web::Result;
use std::str::FromStr;

pub type QueryGet = Vec<(String, String)>;
pub trait QueryGetTrait {
    fn get_string(&self, key: &str) -> Result<String>;
    /// get request get param and parse
    fn get_parse<T>(&self, key: &str) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display;
}

impl QueryGetTrait for Query<QueryGet> {
    fn get_string(&self, key: &str) -> Result<String> {
        return self.0.iter().fold(
            Result::Err(ErrorBadRequest(format!("not find {} param", key))),
            |xx, x| {
                if x.0 == key {
                    Result::Ok(x.1.to_string())
                } else {
                    xx
                }
            },
        );
    }
    /// get request get param and parse
    fn get_parse<T>(&self, key: &str) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display,
    {
        let b = self.get_string(key);
        match b {
            Ok(e) => {
                let b = e.as_str().parse::<T>();
                match b {
                    Ok(e) => Result::Ok(e),
                    Err(e) => {
                        let err = format!("paras {} fail :{}", key, e);
                        Result::Err(ErrorBadRequest(err))
                    }
                }
            }
            Err(e) => Err(e),
        }
    }
}
