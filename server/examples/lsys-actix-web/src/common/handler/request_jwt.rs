use std::{pin::Pin, str::FromStr};

use actix_web::{dev::Payload, FromRequest, HttpRequest};

use lsys_web::common::{
    JsonData, JsonResponse, JsonResult, RequestSessionToken, RequestSessionTokenPaser,
};
use lsys_web::lsys_user::dao::UserAuthToken;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

use async_trait::async_trait;
use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use std::{
    future::Future,
    task::{Context, Poll},
};

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

impl JwtClaims {
    pub fn new(exp: i64, token: String, data: Option<Value>) -> Self {
        Self { exp, token, data }
    }
}

pub struct JwtExtractFut {
    req: HttpRequest,
}

impl Future for JwtExtractFut {
    type Output = Result<JwtQuery, ResponseJson>;
    fn poll(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Self::Output> {
        let res = match self.req.headers().get("Authorization") {
            Some(head) => {
                match head.to_str() {
                    Ok(mut token) => {
                        token = token.trim_start();
                        if !token.starts_with("Bearer ") {
                            Err(JsonResponse::data(
                                JsonData::error().set_sub_code("jwt_parse_header"),
                            )
                            .set_message("not bearer header")
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
                                       // let token_str = token.to_owned();
                                        Ok(JwtQuery {
                                            token_data,
                                        //    token_str,
                                        })
                                    }
                                    Err(e) => Err(match e.kind() {
                                        jsonwebtoken::errors::ErrorKind::InvalidToken
                                        | jsonwebtoken::errors::ErrorKind::InvalidSignature
                                        | jsonwebtoken::errors::ErrorKind::InvalidAlgorithmName
                                        | jsonwebtoken::errors::ErrorKind::RsaFailedSigning
                                        | jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(_)
                                        | jsonwebtoken::errors::ErrorKind::ExpiredSignature
                                        | jsonwebtoken::errors::ErrorKind::MissingAlgorithm
                                        | jsonwebtoken::errors::ErrorKind::InvalidAlgorithm
                                        | jsonwebtoken::errors::ErrorKind::ImmatureSignature
                                        | jsonwebtoken::errors::ErrorKind::InvalidSubject => {
                                            JsonResponse::data(JsonData::error().set_sub_code("jwt_bad_token"))
                                            .set_message(e).into()
                                        }
                                        _ => JsonResponse::data(JsonData::error().set_sub_code("jwt_parse_system"))
                                        .set_message(e)
                                            .into(),
                                    }),
                                }
                                }
                                None => Err(JsonResponse::data(
                                    JsonData::error().set_sub_code("jwt_config"),
                                )
                                .set_message("jwt config not find")
                                .into()),
                            }
                        }
                    }
                    Err(e) => Err(JsonResponse::data(
                        JsonData::error().set_sub_code("jwt_parse_header"),
                    )
                    .set_message(e)
                    .into()),
                }
            }
            None => Err(
                JsonResponse::data(JsonData::error().set_sub_code("jwt_miss_header"))
                    .set_message("jwt miss Authorization header")
                    .into(),
            ),
        };
        Poll::Ready(res)
    }
}

pub struct JwtQueryTokenPaser {}

#[async_trait]
impl RequestSessionTokenPaser<UserAuthToken> for JwtQueryTokenPaser {
    type TD = String;
    async fn parse_user_token(&self, jwt_str: String) -> JsonResult<UserAuthToken> {
        Ok(UserAuthToken::from_str(&jwt_str)?)
    }
}
//jwt 登陆信息实现，服务器端处理跟cookie相同
pub struct JwtQuery {
    pub token_data: TokenData<JwtClaims>,
}

#[async_trait]
impl RequestSessionToken<UserAuthToken> for JwtQuery {
    type L = JwtQueryTokenPaser;
    fn get_paser(&self) -> Self::L {
        JwtQueryTokenPaser {}
    }
    fn get_token_data(&self) -> Option<String> {
        let jstr = self.token_data.claims.token.to_owned();
        if jstr.is_empty() {
            None
        } else {
            Some(jstr)
        }
    }
    fn finish_user_token(&self, _: &UserAuthToken) {}
}

impl JwtQuery {
    #[allow(dead_code)]
    pub fn param<T: DeserializeOwned>(&self) -> Result<T, JsonResponse> {
        match self.token_data.claims.data {
            Some(ref data) => serde_json::value::from_value::<T>(data.clone())
                .map_err(|err| JsonResponse::data(JsonData::error()).set_message(err)),
            None => Err(
                JsonResponse::data(JsonData::error().set_sub_code("jwt_miss_data"))
                    .set_message("data is null"),
            ),
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
