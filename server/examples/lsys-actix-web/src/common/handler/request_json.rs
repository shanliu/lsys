//! Json extractor.
use core::fmt::Debug;
use std::sync::Arc;

use actix_web::dev::{JsonBody, Payload};
use actix_web::FromRequest;
use actix_web::HttpRequest;
use futures_util::future::{FutureExt, LocalBoxFuture};
use lsys_web:: common::JsonData;
use serde::de::DeserializeOwned;
use serde_json::Value;
use tracing::debug;

use crate::error::Error;

#[derive(Debug)]
pub struct JsonQuery(Value);

impl JsonQuery {
    pub fn param<T: DeserializeOwned>(&self) -> Result<T, JsonData> {
        serde_json::value::from_value::<T>(self.0.clone()).map_err(JsonData::error)
    }
}

impl FromRequest for JsonQuery {
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    #[inline]
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let req2 = req.clone();
        let (limit, err, ctype) = req
            .app_data::<JsonConfig>()
            .map(|c| (c.limit, c.ehandler.clone(), c.content_type.clone()))
            .unwrap_or((32768, None, None));

        JsonBody::new(req, payload, ctype.as_deref(), false)
            .limit(limit)
            .map(|res: Result<Value, _>| match res {
                Ok(data) => Ok(JsonQuery(data)),
                Err(e) => Err(Error::from(e)),
            })
            .map(move |res| match res {
                Ok(data) => Ok(data),
                Err(e) => {
                    debug!(
                        "Failed to deserialize Json from payload. \
                         Request path: {}",
                        req2.path()
                    );
                    if let Some(err) = err {
                        Err((*err)(e, &req2))
                    } else {
                        Err(e)
                    }
                }
            })
            .boxed_local()
    }
}

type JsonConfigHandler = Option<Arc<dyn Fn(Error, &HttpRequest) -> actix_web::Error + Send + Sync>>;

#[derive(Clone)]
pub struct JsonConfig {
    limit: usize,
    ehandler: JsonConfigHandler,
    content_type: Option<Arc<dyn Fn(mime::Mime) -> bool + Send + Sync>>,
}

impl JsonConfig {
    /// Change max size of payload. By default max size is 32Kb
    #[allow(dead_code)]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }

    /// Set custom error handler
    #[allow(dead_code)]
    pub fn error_handler<F>(mut self, f: F) -> Self
    where
        F: Fn(Error, &HttpRequest) -> actix_web::Error + Send + Sync + 'static,
    {
        self.ehandler = Some(Arc::new(f));
        self
    }

    /// Set predicate for allowed content types
    #[allow(dead_code)]
    pub fn content_type<F>(mut self, predicate: F) -> Self
    where
        F: Fn(mime::Mime) -> bool + Send + Sync + 'static,
    {
        self.content_type = Some(Arc::new(predicate));
        self
    }
}

impl Default for JsonConfig {
    fn default() -> Self {
        JsonConfig {
            limit: 32768,
            ehandler: None,
            content_type: None,
        }
    }
}
