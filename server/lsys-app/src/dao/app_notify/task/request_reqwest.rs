use crate::dao::{AppError, AppSecretRecrod};
use crate::model::{AppModel, AppNotifyConfigModel, AppNotifyDataModel};
use chrono::{DateTime, Local};

use super::AppNotifyRequest;
use async_trait::async_trait;
use lsys_core::fluent_message;
use reqwest::Client;
use reqwest::{
    header::{HeaderMap, HeaderValue},
    StatusCode,
};
use std::time::Duration;
use tracing::{debug, info};

pub struct AppNotifyRequestReqwest {
    client: Client,
}

impl AppNotifyRequestReqwest {
    pub fn new(timeout: Duration) -> Result<Self, AppError> {
        let client = reqwest::Client::builder();
        let client = client
            .timeout(timeout) //超时 Duration::from_secs(5)
            .build()
            .map_err(|e| {
                AppError::System(fluent_message!("init notify http request fail:{}", e))
            })?;
        Ok(Self { client })
    }
}

#[async_trait]
impl AppNotifyRequest for AppNotifyRequestReqwest {
    async fn exec_request(
        &self,
        app: &AppModel,
        config: &AppNotifyConfigModel,
        record: &AppNotifyDataModel,
        client_secret: &AppSecretRecrod,
    ) -> Result<(), String> {
        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str("application/json;charset=utf-8") {
            headers.insert("Content-Type", value);
        }
        let now: DateTime<Local> = Local::now();
        let timestamp = now.format("%Y-%m-%d %H:%M:%S").to_string();

        let mut params = vec![
            ("client_id", app.client_id.as_str()),
            ("version", "3.0"),
            ("timestamp", timestamp.as_str()),
            ("method", record.notify_method.as_str()),
        ];

        let mut url_params = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params.clone())
            .finish();

        let payload = record.notify_payload.trim();
        if !payload.is_empty() {
            url_params += payload;
        }
        url_params += client_secret.secret_data.as_str();
        let digest = md5::compute(url_params.as_bytes());
        let hash = format!("{:x}", digest);

        params.push(("sign", hash.as_str()));

        let param_str = url::form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();

        let mut call_url = config.call_url.to_owned();
        if !config.call_url.contains('?') {
            call_url += "?";
        }
        let trimmed = call_url.trim();
        if !trimmed.ends_with('&') && !trimmed.ends_with('?') {
            call_url += "&";
        }
        call_url += param_str.as_str();

        debug!("notify url:{}", &call_url);

        let request = self.client.post(&call_url).body(payload.to_string());

        match request.send().await {
            Ok(resp) => {
                if resp.status() == StatusCode::OK {
                    debug!("http notify succ:{} [{}]", &call_url, resp.status());
                    return Ok(());
                } else {
                    use futures::StreamExt;
                    let mut buffer = Vec::new();
                    let http_code = resp.status();
                    let mut stream = resp.bytes_stream().take(256);
                    while let Some(item) = stream.next().await {
                        match item {
                            Ok(st) => {
                                buffer.extend_from_slice(&st);
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                    let s = format!(
                        "Code:{} Res:{}\nUrl:{}",
                        http_code.as_u16(),
                        String::from_utf8_lossy(&buffer),
                        call_url,
                    );
                    info!("http notify fail: {} [{}]", &call_url, &s);
                    Err(s)
                }
            }
            Err(err) => Err(err.to_string()),
        }
    }
}
