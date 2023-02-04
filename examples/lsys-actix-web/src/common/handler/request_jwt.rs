use std::{pin::Pin, str::FromStr};

use actix_web::{dev::Payload, FromRequest, HttpRequest};

use lsys_user::dao::auth::{SessionToken, UserAuthTokenData};
use lsys_web::{dao::RequestToken, JsonData};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use std::{
    future::Future,
    task::{Context, Poll},
};

use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};

use super::ResponseJson;

#[derive(Clone)]
pub struct JwtQueryConfig {
    pub decode_key: DecodingKey,
    pub validation: Validation,
}

impl JwtQueryConfig {
    /// Set maximum accepted payload size. By default this limit is 16kB.
    pub fn new(decode_key: DecodingKey, validation: Validation) -> Self {
        Self {
            decode_key,
            validation,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct JwtClaims {
    pub exp: i64,
    pub token: String,
    data: Option<Value>,
}

pub struct JwtExtractFut {
    req: HttpRequest,
}

impl Future for JwtExtractFut {
    type Output = Result<JwtQuery, ResponseJson>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let res = match self.req.headers().get("Authorization") {
            Some(head) => match head.to_str() {
                Ok(mut token) => {
                    token = token.trim_start();
                    if !token.starts_with("Bearer ") {
                        Err(JsonData::message("not bearer header")
                            .set_sub_code("jwt_parse_header")
                            .into())
                    } else {
                        token = &token[7..];
                        token = token.trim();

                        let config_opt = self.req.app_data::<JwtQueryConfig>();
                        match config_opt {
                            Some(config) => {
                                let token_data_opt = decode::<JwtClaims>(
                                    token,
                                    &config.decode_key,
                                    &config.validation,
                                );
                                match token_data_opt {
                                    Ok(token_data) => {
                                        let token_str = token.to_owned();
                                        Ok(JwtQuery {
                                            token_data,
                                            token_str,
                                        })
                                    }
                                    Err(e) => Err(JsonData::error(e)
                                        .set_sub_code("jwt_parse_header")
                                        .into()),
                                }
                            }
                            None => Err(JsonData::message("jwt config not find")
                                .set_sub_code("jwt_config")
                                .into()),
                        }
                    }
                }
                Err(e) => Err(JsonData::error(e).set_sub_code("jwt_parse_header").into()),
            },
            None => Err(JsonData::message_error("jwt miss Authorization header")
                .set_sub_code("jwt_miss_header")
                .into()),
        };
        Poll::Ready(res)
    }
}

//jwt 登陆信息实现，服务器端处理跟cookie相同
pub struct JwtQuery {
    pub token_data: TokenData<JwtClaims>,
    pub token_str: String,
}

impl RequestToken<UserAuthTokenData> for JwtQuery {
    fn get_user_token<'t>(&self) -> SessionToken<UserAuthTokenData> {
        SessionToken::<UserAuthTokenData>::from_str(self.token_data.claims.token.as_str())
            .unwrap_or_default()
    }
    fn is_refresh(&self, _token: &SessionToken<UserAuthTokenData>) -> bool {
        false
    }
    fn refresh_user_token(&self, _token: &SessionToken<UserAuthTokenData>) {
        unimplemented!("not support refresh");
    }
}

impl JwtQuery {
    #[allow(dead_code)]
    pub fn param<T: DeserializeOwned>(&self) -> Result<T, JsonData> {
        match self.token_data.claims.data {
            Some(ref data) => {
                serde_json::value::from_value::<T>(data.clone()).map_err(JsonData::error)
            }
            None => Err(JsonData::message_error("data is null").set_sub_code("jwt_miss_data")),
        }
    }
}

impl FromRequest for JwtQuery {
    type Error = ResponseJson;
    type Future = JwtExtractFut;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        JwtExtractFut { req: req.clone() }
    }
}
