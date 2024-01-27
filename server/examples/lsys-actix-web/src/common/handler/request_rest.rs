use std::collections::BTreeMap;

use std::ops::Deref;
use std::sync::Arc;
use std::{
    future::Future,
    task::{Context, Poll},
};
use std::{pin::Pin, rc::Rc};

use actix_web::web::{Data, JsonBody};
use actix_web::{dev::Payload, Error, FromRequest, HttpRequest};
use futures_util::{ready, FutureExt};
use lsys_app::dao::session::RestAuthTokenData;

use lsys_core::RequestEnv;
use lsys_user::dao::auth::SessionToken;

use lsys_web::dao::{RequestDao, RequestSessionToken, WebDao};
use lsys_web::JsonData;

use serde::{de::DeserializeOwned, Deserialize};
use serde_json::Value;
use tracing::info;

use super::ResponseJson;

#[derive(Deserialize)]
pub struct RestGet {
    pub app_id: String,
    pub version: String,
    pub timestamp: String,
    pub sign: String,
    pub payload: Option<String>,
    pub request_ip: Option<String>,
    pub method: Option<String>,
    pub token: Option<String>,
    pub lang: Option<String>,
}

pub struct RestRfc {
    pub app_id: String,
    pub version: String,
    pub timestamp: String,
    pub sign: String,
    pub request_lang: Option<String>,
    pub payload: Option<Value>,
    pub request_ip: Option<String>,
    pub request_id: Option<String>,
    pub method: Option<String>,
    pub token: Option<String>,
}

type RestKeyGet =
    Box<dyn Fn(String, Data<WebDao>) -> Pin<Box<dyn Future<Output = Result<String, String>>>>>;

type RestKeyGetOption = Option<Rc<RestKeyGet>>;

async fn check_sign(
    data: &RestRfc,
    key_fn: &RestKeyGetOption,
    app_data: Data<WebDao>,
) -> Result<(), JsonData> {
    match key_fn {
        Some(kfn) => {
            let key_res = kfn.as_ref()(data.app_id.clone(), app_data.clone())
                .as_mut()
                .await;
            match key_res {
                Ok(app_key) => {
                    let mut map_data = BTreeMap::from([
                        ("app_id", &data.app_id),
                        ("version", &data.version),
                        ("timestamp", &data.timestamp),
                    ]);
                    if let Some(ref request_ip) = data.request_ip {
                        map_data.insert("request_ip", request_ip);
                    }
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
                    let mut url_data = encoded.finish();
                    if let Some(ref body) = data.payload {
                        url_data += body.to_string().as_str();
                    }
                    url_data += app_key.as_str();
                    let digest = md5::compute(url_data.as_bytes());
                    let hash = format!("{:x}", digest);

                    if hash != data.sign {
                        return Err(JsonData::message("sign is wrong").set_sub_code("rest_sign"));
                    }
                    Ok(())
                }
                Err(err) => Err(JsonData::message_error(format!("{}:{}", data.app_id, err))
                    .set_sub_code("rest_sign_key")),
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

type RestExtractBody = Option<Pin<Box<dyn Future<Output = Result<Value, JsonData>>>>>;
type RestExtractFuture = Option<Pin<Box<dyn Future<Output = Result<RestQuery, ResponseJson>>>>>;

pub struct RestExtractFut {
    rest_dao: RestWebDao,
    rfc: Option<RestRfc>,
    body: RestExtractBody,
    future: RestExtractFuture,
}

impl Future for RestExtractFut {
    type Output = Result<RestQuery, ResponseJson>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(ref mut future) = self.future {
            return future.as_mut().poll(cx);
        }
        let rest_dao = self.rest_dao.clone();
        match rest_dao {
            RestWebDao::Err(e) => Poll::Ready(Err(e.into())),
            RestWebDao::AppDat(app_dao, key_fn) => {
                match &mut self.body {
                    Some(body_data) => {
                        let body_res = ready!(body_data.as_mut().poll(cx));
                        let rfc_data = self.rfc.take();
                        match rfc_data {
                            Some(mut rfc) => {
                                let mut future = Box::pin(async move {
                                    match body_res {
                                        Ok(body) => {
                                            rfc.payload = Some(body);
                                            check_sign(&rfc, &key_fn, app_dao.to_owned()).await?;
                                            Ok(RestQuery::new(
                                                app_dao.into_inner(),
                                                RequestEnv::new(
                                                    rfc.request_lang.clone(),
                                                    rfc.request_ip.clone(),
                                                    rfc.request_id.clone(),
                                                    None,
                                                ),
                                                rfc,
                                            ))
                                        }
                                        Err(err) => Err(err.into()),
                                    }
                                });
                                match future.as_mut().poll(cx) {
                                    Poll::Ready(item) => Poll::Ready(Ok(item?)),
                                    Poll::Pending => {
                                        self.get_mut().future = Some(future);
                                        Poll::Pending
                                    }
                                }
                            }
                            None => Poll::Ready(Err(JsonData::message_error("rfc is take").into())), //理论上不会进这里
                        }
                    }
                    None => {
                        let rfc_data = self.rfc.take();
                        match rfc_data {
                            Some(rfc) => {
                                let mut future = Box::pin(async move {
                                    check_sign(&rfc, &key_fn, app_dao.to_owned()).await?;
                                    Ok(RestQuery::new(
                                        app_dao.into_inner(),
                                        RequestEnv::new(
                                            rfc.request_lang.clone(),
                                            rfc.request_ip.clone(),
                                            rfc.request_id.clone(),
                                            None,
                                        ),
                                        rfc,
                                    ))
                                });
                                match future.as_mut().poll(cx) {
                                    Poll::Ready(item) => Poll::Ready(Ok(item?)),
                                    Poll::Pending => {
                                        self.get_mut().future = Some(future);
                                        Poll::Pending
                                    }
                                }
                            }
                            None => Poll::Ready(Err(JsonData::message_error("rfc is take").into())), //理论上不会进这里
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RestQueryConfig {
    limit: usize,
    app_key: RestKeyGetOption,
}

impl RestQueryConfig {
    /// Set maximum accepted payload size. By default this limit is 16kB.
    #[allow(dead_code)]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
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
    app_key: None,
};

impl Default for RestQueryConfig {
    fn default() -> Self {
        DEFAULT_CONFIG
    }
}

pub struct RestQuery {
    inner: RequestDao,
    pub rfc: RestRfc,
}

impl Deref for RestQuery {
    type Target = RequestDao;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl RequestSessionToken<RestAuthTokenData> for RestQuery {
    fn get_user_token(&self) -> SessionToken<RestAuthTokenData> {
        self.rfc
            .token
            .as_ref()
            .map(|e| {
                if e.is_empty() {
                    SessionToken::default()
                } else {
                    let data = RestAuthTokenData {
                        client_id: self.rfc.app_id.clone(),
                        token: e.to_owned(),
                    };
                    SessionToken::from_data(Some(data))
                }
            })
            .unwrap_or_default()
    }
    fn is_refresh(&self, _token: &SessionToken<RestAuthTokenData>) -> bool {
        false
    }
    fn refresh_user_token(&self, _token: &SessionToken<RestAuthTokenData>) {
        unimplemented!("not support refresh");
    }
}

impl RestQuery {
    pub fn new(web_dao: Arc<WebDao>, req_env: RequestEnv, rfc: RestRfc) -> Self {
        Self {
            inner: RequestDao::new(web_dao, req_env),
            rfc,
        }
    }
    pub fn param<T: DeserializeOwned>(&mut self) -> Result<T, JsonData> {
        let body_data = self.rfc.payload.take();
        match body_data {
            Some(body) => serde_json::from_value::<T>(body)
                .map_err(|e| JsonData::error(e).set_sub_code("rest_param_wrong")),
            None => {
                Err(JsonData::message_error("param is empty or take")
                    .set_sub_code("rest_param_empty"))
            }
        }
    }
    // pub fn clone_param<T: DeserializeOwned>(&self) -> Result<T, JsonData> {
    //     match &self.rfc.body {
    //         Some(body) => serde_json::from_value::<T>(body.to_owned())
    //             .map_err(|e| JsonData::error(e).set_sub_code("rest_param_wrong")),
    //         None => {
    //             Err(JsonData::message_error("param is empty or take")
    //                 .set_sub_code("rest_param_empty"))
    //         }
    //     }
    // }
}

impl FromRequest for RestQuery {
    type Error = ResponseJson;
    type Future = RestExtractFut;
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let RestQueryConfig { limit, app_key } = req
            .app_data::<RestQueryConfig>()
            .or_else(|| req.app_data::<RestQueryConfig>())
            .unwrap_or(&RestQueryConfig::default())
            .to_owned();
        let json_req = req
            .headers()
            .get("Content-type")
            .map(|e| e.to_str().unwrap_or_default())
            .unwrap_or_default()
            .contains("application/json");
        let request_id = req
            .headers()
            .get("X-Request-ID")
            .map(|e| e.to_str().unwrap_or_default().to_string());

        let (rest_dao, rfc) = match req.app_data::<Data<WebDao>>() {
            Some(app_dao) => match serde_urlencoded::from_str::<RestGet>(req.query_string()) {
                Ok(get_param) => {
                    let rest_dao = RestWebDao::AppDat(app_dao.clone(), app_key);
                    let mut rfc = RestRfc {
                        request_id,
                        request_lang: get_param.lang,
                        app_id: get_param.app_id,
                        version: get_param.version,
                        timestamp: get_param.timestamp,
                        sign: get_param.sign,
                        payload: None,
                        request_ip: get_param.request_ip,
                        method: get_param.method,
                        token: get_param.token,
                    };
                    if !json_req {
                        if let Some(pl) = get_param.payload {
                            if !pl.is_empty() {
                                match serde_json::from_str::<Value>(pl.as_str()) {
                                    Ok(val) => {
                                        rfc.payload = Some(val);
                                        (rest_dao, Some(rfc))
                                    }
                                    Err(err) => (
                                        RestWebDao::Err(
                                            JsonData::error(err).set_sub_code("rest_payload"),
                                        ),
                                        None,
                                    ),
                                }
                            } else {
                                (rest_dao, Some(rfc))
                            }
                        } else {
                            (rest_dao, Some(rfc))
                        }
                    } else {
                        (rest_dao, Some(rfc))
                    }
                }
                Err(err) => (
                    RestWebDao::Err(JsonData::error(err).set_sub_code("rest_parse")),
                    None,
                ),
            },
            None => (
                RestWebDao::Err(
                    JsonData::message_error("web dao not reg").set_sub_code("rest_config"),
                ),
                None,
            ),
        };
        let body = if json_req {
            let path = req.path().to_string();
            let data = JsonBody::new(req, payload, None, false)
                .limit(limit)
                .map(|res: Result<Value, _>| match res {
                    Ok(data) => Ok(data),
                    Err(e) => Err(Error::from(e)),
                })
                .map(move |res| match res {
                    Ok(data) => Ok(data),
                    Err(e) => {
                        info!(
                            "Failed to deserialize Json from payload. Request path: {}",
                            path
                        );
                        Err(JsonData::error(e).set_sub_code("rest_payload"))
                    }
                })
                .boxed_local();

            Some(data)
        } else {
            None
        };
        RestExtractFut {
            rest_dao,
            body,
            rfc,
            future: None,
        }
    }
}
