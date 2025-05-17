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

use lsys_app::dao::RestAuthToken;
use lsys_core::{IntoFluentMessage, RequestEnv};

use lsys_web::common::{JsonData, JsonResponse, RequestDao, RequestSessionToken};
use lsys_web::dao::WebDao;

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
    Box<dyn Fn(String, Data<WebDao>) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>>>>>;

type RestKeyGetOption = Option<Rc<RestKeyGet>>;

async fn check_sign(
    data: &RestRfc,
    key_fn: &RestKeyGetOption,
    app_data: Data<WebDao>,
) -> Result<(), JsonResponse> {
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

                    for key_tmp in app_key {
                        let hash_data = url_data.clone() + key_tmp.as_str();
                        // dbg!(&url_data);
                        let digest = md5::compute(hash_data.as_bytes());
                        let hash = format!("{:x}", digest);
                        if hash == data.sign {
                            return Ok(());
                        }
                    }
                    Err(
                        JsonResponse::data(JsonData::error().set_sub_code("rest_sign"))
                            .set_message("sign is wrong"),
                    )
                }
                Err(err) => Err(JsonResponse::data(
                    JsonData::error().set_sub_code("rest_sign_key"),
                )
                .set_message(err)),
            }
        }
        None => Ok(()),
    }
}

#[derive(Clone)]
enum RestWebDao {
    Err(JsonResponse),
    AppDat(Data<WebDao>, RestKeyGetOption),
}

type RestExtractBody = Option<Pin<Box<dyn Future<Output = Result<Value, JsonResponse>>>>>;
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
                                            match RequestEnv::new(
                                                rfc.request_lang.as_deref(),
                                                rfc.request_ip.as_deref(),
                                                rfc.request_id.as_deref(),
                                                None,
                                                None,
                                            ) {
                                                Ok(env) => Ok(RestQuery::new(
                                                    app_dao.into_inner(),
                                                    env,
                                                    rfc,
                                                )),
                                                Err(verr) => Err(JsonResponse::data(
                                                    JsonData::default()
                                                        .set_sub_code("env_valid_err")
                                                        .set_code(400),
                                                )
                                                .set_message(
                                                    verr.to_fluent_message().default_format(),
                                                )
                                                .into()),
                                            }
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
                            None => Poll::Ready(Err(JsonResponse::data(JsonData::error())
                                .set_message("rfc is take")
                                .into())), //理论上不会进这里
                        }
                    }
                    None => {
                        let rfc_data = self.rfc.take();
                        match rfc_data {
                            Some(rfc) => {
                                let mut future = Box::pin(async move {
                                    check_sign(&rfc, &key_fn, app_dao.to_owned()).await?;
                                    match RequestEnv::new(
                                        rfc.request_lang.as_deref(),
                                        rfc.request_ip.as_deref(),
                                        rfc.request_id.as_deref(),
                                        None,
                                        None,
                                    ) {
                                        Ok(env) => {
                                            Ok(RestQuery::new(app_dao.into_inner(), env, rfc))
                                        }
                                        Err(verr) => Err(JsonResponse::data(
                                            JsonData::default()
                                                .set_sub_code("env_valid_err")
                                                .set_code(400),
                                        )
                                        .set_message(verr.to_fluent_message().default_format())
                                        .into()),
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
                            None => Poll::Ready(Err(JsonResponse::data(JsonData::error())
                                .set_message("rfc is take")
                                .into())), //理论上不会进这里
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

impl RequestSessionToken<RestAuthToken> for RestQuery {
    fn get_user_token(&self) -> RestAuthToken {
        self.rfc
            .token
            .as_ref()
            .map(|e| {
                if e.is_empty() {
                    RestAuthToken::default()
                } else {
                    RestAuthToken {
                        client_id: self.rfc.app_id.clone(),
                        token: e.to_owned(),
                    }
                }
            })
            .unwrap_or_default()
    }
    fn is_refresh(&self, _token: &RestAuthToken) -> bool {
        false
    }
    fn refresh_user_token(&self, _token: &RestAuthToken) {
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
    pub fn param<T: DeserializeOwned>(&self) -> Result<T, JsonResponse> {
        match self.rfc.payload {
            Some(ref body) => serde_json::from_value::<T>(body.to_owned()).map_err(|e| {
                JsonResponse::data(JsonData::error().set_sub_code("rest_param_wrong"))
                    .set_message(e)
            }),
            None => Err(
                JsonResponse::data(JsonData::error().set_sub_code("rest_param_empty"))
                    .set_message("param is empty or take"),
            ),
        }
    }
    pub async fn get_app(&self) -> Result<lsys_app::model::AppModel, JsonResponse> {
        self.web_dao
            .web_app
            .app_dao
            .app
            .cache()
            .find_by_client_id(&self.rfc.app_id)
            .await
            .map_err(|e| self.fluent_error_json_response(&e.into()))
    }
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
                                            JsonResponse::data(
                                                JsonData::error().set_sub_code("rest_payload"),
                                            )
                                            .set_message(err),
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
                    RestWebDao::Err(
                        JsonResponse::data(JsonData::error().set_sub_code("rest_parse"))
                            .set_message(err),
                    ),
                    None,
                ),
            },
            None => (
                RestWebDao::Err(
                    JsonResponse::data(JsonData::error().set_sub_code("rest_config"))
                        .set_message("web dao not reg"),
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
                        Err(
                            JsonResponse::data(JsonData::error().set_sub_code("rest_payload"))
                                .set_message(e),
                        )
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
