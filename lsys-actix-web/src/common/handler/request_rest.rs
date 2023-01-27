use std::collections::BTreeMap;

use std::{
    future::Future,
    task::{Context, Poll},
};
use std::{pin::Pin, rc::Rc};

use actix_web::web::Data;
use actix_web::{
    dev::Payload, error::UrlencodedError, web::UrlEncoded, Error, FromRequest, HttpRequest,
};
use futures_util::ready;
use lsys_app::dao::session::RestAuthTokenData;

use lsys_user::dao::auth::SessionToken;

use lsys_web::dao::{RequestToken, WebDao};
use lsys_web::JsonData;

use serde::{de::DeserializeOwned, Deserialize};

use super::ResponseJson;

#[derive(Deserialize)]
pub struct RestRfc {
    pub app: String,
    pub version: String,
    pub timestamp: String,
    pub content: String,
    pub sign: String,
    pub method: Option<String>,
    pub token: Option<String>,
}

enum RestFormData {
    Fut(UrlEncoded<RestRfc>),
    Query(String),
}

type RestKeyGet =
    Box<dyn Fn(String, Data<WebDao>) -> Pin<Box<dyn Future<Output = Result<String, String>>>>>;

type RestKeyGetOption = Option<Rc<RestKeyGet>>;

async fn check_sign(
    data: &RestRfc,
    key_fn: RestKeyGetOption,
    app_data: Data<WebDao>,
) -> Result<(), JsonData> {
    match key_fn {
        Some(kfn) => {
            let key_res = kfn.as_ref()(data.app.clone(), app_data.clone())
                .as_mut()
                .await;
            match key_res {
                Ok(app_key) => {
                    let mut map_data = BTreeMap::from([
                        ("app", &data.app),
                        ("version", &data.version),
                        ("timestamp", &data.timestamp),
                        ("content", &data.content),
                    ]);
                    if let Some(ref method) = data.method {
                        map_data.insert("method", method);
                    }
                    if let Some(ref token) = data.token {
                        map_data.insert("token", token);
                    }
                    let mut encoded = &mut form_urlencoded::Serializer::new(String::new());
                    for (e0, e1) in map_data.into_iter() {
                        encoded = encoded.append_pair(e0, e1.as_str())
                    }
                    let url_data = encoded.finish() + app_key.as_str();
                    let digest = md5::compute(url_data.as_bytes());
                    let hash = format!("{:x}", digest);

                    if hash != data.sign {
                        return Err(JsonData::message_error("sign is wrong")
                            .set_code(400)
                            .set_sub_code("rest_sign"));
                    }

                    //check token
                    // if let Some(ref token)=data.token{
                    //     app_data.user.auth.set_token_str(token.as_str());
                    // }

                    Ok(())
                }
                Err(err) => Err(JsonData::message_error(format!("{}:{}", data.app, err))),
            }
        }
        None => Ok(()),
    }
}

#[derive(Clone)]
enum RestWebDao {
    Err(JsonData),
    AppDat(Data<WebDao>, RestKeyGetOption),
}

type FormErrHandler = Option<Rc<dyn Fn(UrlencodedError, &HttpRequest) -> Error>>;

pub struct RestExtractFut {
    web_dao: RestWebDao,
    fut: RestFormData,
    err_handler: FormErrHandler,
    req: HttpRequest,
    #[allow(clippy::type_complexity)]
    future: Option<Pin<Box<dyn Future<Output = Result<RestRfc, JsonData>>>>>,
}

impl Future for RestExtractFut {
    type Output = Result<RestQuery, ResponseJson>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ref mut future) = self.future {
            let ret = match future.as_mut().poll(cx) {
                Poll::Ready(item) => Ok(RestQuery { rfc: item? }),
                Poll::Pending => {
                    return Poll::Pending;
                }
            };
            return Poll::Ready(ret);
        }
        let app_dao = self.web_dao.clone();
        let err_handler = self.err_handler.clone();
        let reg = self.req.clone();
        match app_dao {
            RestWebDao::Err(e) => Poll::Ready(Err(e.into())),
            RestWebDao::AppDat(app_dao, key_fn) => {
                let err_fn = &err_handler;
                let req = &reg;
                match &mut self.fut {
                    RestFormData::Fut(ref mut dat) => {
                        let p = Pin::new(dat).poll(cx);
                        let res = ready!(p);
                        let res = match res {
                            Err(err) => match err_fn {
                                Some(err_handler) => {
                                    let err = (err_handler)(err, req);
                                    Err(JsonData::error(err).into())
                                }
                                None => Err(JsonData::error(err).into()),
                            },
                            Ok(item) => {
                                let mut future = Box::pin(async move {
                                    check_sign(&item, key_fn, app_dao).await.map(|_| item)
                                });
                                let ret = match future.as_mut().poll(cx) {
                                    Poll::Ready(item) => Ok(RestQuery { rfc: item? }),
                                    Poll::Pending => {
                                        self.get_mut().future = Some(future);
                                        return Poll::Pending;
                                    }
                                };
                                ret
                            }
                        };
                        Poll::Ready(res)
                    }
                    RestFormData::Query(queru_str) => {
                        let wrap = serde_urlencoded::from_str::<RestRfc>(queru_str)
                            .map_err(JsonData::error)?;

                        let mut future = Box::pin(async move {
                            check_sign(&wrap, key_fn, app_dao).await.map(|_| wrap)
                        });
                        let ret = match future.as_mut().poll(cx) {
                            Poll::Ready(item) => Ok(RestQuery { rfc: item? }),
                            Poll::Pending => {
                                self.get_mut().future = Some(future);
                                return Poll::Pending;
                            }
                        };
                        Poll::Ready(ret)
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RestQueryConfig {
    limit: usize,
    err_handler: FormErrHandler,
    app_key: RestKeyGetOption,
}

impl RestQueryConfig {
    /// Set maximum accepted payload size. By default this limit is 16kB.
    #[allow(dead_code)]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    /// Set custom error handler
    #[allow(dead_code)]
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(UrlencodedError, &HttpRequest) -> Error + 'static,
    {
        self.err_handler = Some(Rc::new(f));
        self
    }
    #[allow(dead_code)]
    pub fn app_key_fn(mut self, f: RestKeyGet) -> Self {
        self.app_key = Some(Rc::new(f));
        self
    }
}

/// Allow shared refs used as default.
const DEFAULT_CONFIG: RestQueryConfig = RestQueryConfig {
    limit: 16_384, // 2^14 bytes (~16kB)
    err_handler: None,
    app_key: None,
};

impl Default for RestQueryConfig {
    fn default() -> Self {
        DEFAULT_CONFIG
    }
}

fn check_app_dao(req: &HttpRequest, app_key: RestKeyGetOption) -> RestWebDao {
    if let Some(app_dao) = req.app_data::<Data<WebDao>>() {
        return RestWebDao::AppDat(app_dao.clone(), app_key);
    }
    RestWebDao::Err(JsonData::message_error("sign not config").set_sub_code("rest_config"))
}

pub struct RestQuery {
    pub rfc: RestRfc,
}

impl RequestToken<RestAuthTokenData> for RestQuery {
    fn get_user_token(&self) -> SessionToken<RestAuthTokenData> {
        self.rfc
            .token
            .as_ref()
            .map(|e| {
                if e.is_empty() {
                    SessionToken::default()
                } else {
                    let data = RestAuthTokenData {
                        client_id: self.rfc.app.clone(),
                        token: e.to_owned(),
                    };
                    SessionToken::from_data(Some(data))
                }
            })
            .unwrap_or_else(SessionToken::<RestAuthTokenData>::default)
    }
    fn is_refresh(&self, _token: &SessionToken<RestAuthTokenData>) -> bool {
        false
    }
    fn refresh_user_token(&self, _token: &SessionToken<RestAuthTokenData>) {
        unimplemented!("not support refresh");
    }
}

impl RestQuery {
    pub fn param<T: DeserializeOwned>(&self) -> Result<T, JsonData> {
        serde_json::from_slice::<T>(self.rfc.content.as_bytes()).map_err(JsonData::error)
    }
}

impl FromRequest for RestQuery {
    type Error = ResponseJson;
    type Future = RestExtractFut;
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        if req
            .headers()
            .get("Content-type")
            .map(|e| e.to_str().unwrap_or_default())
            .unwrap_or_default()
            .contains("application/x-www-form-urlencoded")
        {
            let RestQueryConfig {
                limit,
                err_handler,
                app_key,
            } = req
                .app_data::<RestQueryConfig>()
                .or_else(|| req.app_data::<RestQueryConfig>())
                .unwrap_or(&RestQueryConfig::default())
                .to_owned();
            RestExtractFut {
                web_dao: check_app_dao(req, app_key),
                fut: RestFormData::Fut(UrlEncoded::<RestRfc>::new(req, payload).limit(limit)),
                req: req.clone(),
                err_handler,
                future: None,
            }
        } else {
            let RestQueryConfig {
                limit: _,
                err_handler,
                app_key,
            } = req
                .app_data::<RestQueryConfig>()
                .or_else(|| req.app_data::<RestQueryConfig>())
                .unwrap_or(&RestQueryConfig::default())
                .to_owned();
            RestExtractFut {
                web_dao: check_app_dao(req, app_key),
                fut: RestFormData::Query(req.query_string().to_owned()),
                req: req.clone(),
                err_handler,
                future: None,
            }
        }
    }
}
