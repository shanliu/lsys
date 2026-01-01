use std::time::Duration;

use reqwest::{Method, StatusCode, Url};
use serde::Deserialize;
use tracing::{debug, warn};
//lib

pub struct WeChatLib {
    appid: String,
    secret: String,
    agent_id: Option<i32>,
}

static SNSAPI_USERINFO: &str = "snsapi_userinfo";
static GRANT_TYPE: &str = "grant_type";
static CODE: &str = "code";
static APPID: &str = "appid";
static SECRET: &str = "secret";

pub static LANG: &str = "lang";
pub static ZH_CN: &str = "zh_CN";
pub static ACCESS_TOKEN: &str = "access_token";
pub static OPENID: &str = "openid";

impl WeChatLib {
    pub fn new(appid: &str, secret: &str, agent_id: Option<i32>) -> Self {
        Self {
            appid: appid.to_owned(),
            secret: secret.to_owned(),
            agent_id,
        }
    }
    pub fn build_authorization_url(&self, redirect_uri: &str, state: Option<&str>) -> String {
        let mut url = format!(
            "{}?appid={}&redirect_uri={}&response_type=code&scope={}",
            "https://open.weixin.qq.com/connect/oauth2/authorize",
            &self.appid,
            urlencoding::encode(redirect_uri),
            SNSAPI_USERINFO
        );
        if self.agent_id.unwrap_or_default() > 0 {
            url.push_str("&agentid=");
            url.push_str(&self.agent_id.to_owned().unwrap_or_default().to_string());
        }
        if let Some(state) = state {
            url.push_str("&state=");
            url.push_str(state);
        }
        url.push_str("#wechat_redirect");
        url
    }
    pub async fn oauth2_token(
        &self,
        code: &str,
    ) -> Result<WechatMpOauth2AccessTokenResponse, String> {
        let params = vec![
            (GRANT_TYPE.to_string(), "authorization_code".to_string()),
            (CODE.to_string(), code.to_string()),
            (APPID.to_string(), self.appid.to_string()),
            (SECRET.to_string(), self.secret.to_string()),
        ];
        let res = self
            .get("https://api.weixin.qq.com/sns/oauth2/access_token", params)
            .await?;
        let resp = serde_json::from_str::<WechatMpOauth2AccessTokenResponse>(&res)
            .map_err(|e| e.to_string())?;
        Ok(resp)
    }
    pub async fn oauth2_userinfo(
        &self,
        access_token: &str,
        openid: &str,
    ) -> Result<WechatMpOauth2UserInfo, String> {
        let params = vec![
            (ACCESS_TOKEN.to_string(), access_token.to_string()),
            (OPENID.to_string(), openid.to_string()),
            (LANG.to_string(), ZH_CN.to_string()),
        ];
        let res = self
            .get("https://api.weixin.qq.com/sns/userinfo", params)
            .await?;
        let resp =
            serde_json::from_str::<WechatMpOauth2UserInfo>(&res).map_err(|e| e.to_string())?;
        Ok(resp)
    }
    pub async fn get(
        &self,
        http_url: &str,
        params: Vec<(String, String)>,
    ) -> Result<String, String> {
        let mut http_url = Url::parse(http_url).map_err(|e| e.to_string())?;
        http_url.query_pairs_mut().extend_pairs(params.into_iter());
        let client = reqwest::Client::builder();
        let client = client
            .timeout(Duration::from_secs(60))
            .build()
            .map_err(|e| e.to_string())?;
        let request = client.request(Method::GET, http_url.to_owned());
        debug!("wechat oauth url: {}", http_url.as_str(),);
        let result = request.send().await.map_err(|e| e.to_string())?;
        let status = result.status();
        let data = result.bytes().await.map_err(|e| e.to_string())?;
        let res = unsafe { String::from_utf8_unchecked(data.to_vec()) };
        if status != StatusCode::OK {
            warn!("wechat oauth fail response: {}", &res);
        } else {
            debug!("wechat oauth response: {}", &res);
        }
        let resp = serde_json::from_str::<WechatCommonResponse>(&res).map_err(|e| e.to_string())?;
        if !resp.is_success() {
            return Err(format!("oauth get fail:{}", resp.errmsg.unwrap_or(res)));
        }
        Ok(res)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WechatCommonResponse {
    pub errcode: Option<i64>,
    pub errmsg: Option<String>,
}
impl WechatCommonResponse {
    pub fn is_success(&self) -> bool {
        self.errcode.unwrap_or(0) == 0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WechatMpOauth2AccessTokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub openid: String,
    pub scope: String,
    pub expires_in: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WechatMpOauth2UserInfo {
    pub openid: String,
    pub nickname: String,
    pub sex: u8,
    pub city: String,
    pub province: String,
    pub country: String,
    pub headimgurl: String,
    pub unionid: Option<String>,
}
