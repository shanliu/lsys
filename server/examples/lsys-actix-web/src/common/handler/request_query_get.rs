use actix_web::error::ErrorBadRequest;
use actix_web::web::Query;
use actix_web::Result;
use std::str::FromStr;

pub type QueryGet = Vec<(String, String)>;
pub trait QueryGetTrait {
    #[allow(unused)]
    fn get_string(&self, key: &str) -> Result<String>;
    /// get request get param and parse
    #[allow(unused)]
    fn get_parse<T>(&self, key: &str) -> Result<T>
    where
        T: FromStr,
        T::Err: std::fmt::Display;
}

impl QueryGetTrait for Query<QueryGet> {
    fn get_string(&self, key: &str) -> Result<String> {
        for tmp in self.0.iter() {
            if tmp.0 == key {
                return Ok(tmp.1.to_owned());
            }
        }
        Err(ErrorBadRequest(format!("not find {} param", key)))
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
