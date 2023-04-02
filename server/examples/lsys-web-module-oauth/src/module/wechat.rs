use async_trait::async_trait;
// TODO  替换掉这个库,有问题不升级...
use labrador::{SimpleStorage, WechatCpClient, WechatMpClient};

use lsys_web::{
    dao::user::WebUser,
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginData, OauthLoginParam},
    JsonData, JsonResult,
};
use rand::seq::SliceRandom;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

pub const OAUTH_TYPE_WECHAT: &str = "wechat";

fn state_rand(len: usize) -> String {
    const BASE_STR: &str = "0123456789";
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        BASE_STR
            .as_bytes()
            .choose_multiple(&mut rng, len)
            .cloned()
            .collect(),
    )
    .unwrap_or_default()
}
fn state_key(state: &str) -> String {
    format!("wechat-{}", state)
}
fn login_data_key(state: &str) -> String {
    format!("wechat-data-{}", state)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WechatExternalData {}

pub struct WechatLoginParam {
    pub callback_url: String,
    pub state: String,
}
impl OauthLoginParam for WechatLoginParam {}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WechatCallbackParam {
    pub code: String,
    pub state: String,
}
impl OauthCallbackParam for WechatCallbackParam {}

pub struct WechatLogin {
    app_id: String,
    app_secret: String,
    config: String,
    rand_length: usize,
    timeout: usize,  //state保存时间
    timeskip: usize, //剩余多少秒时加载下一个二维码
}

impl WechatLogin {
    pub fn new(app_id: String, app_secret: String, config: String) -> Self {
        Self {
            app_id,
            app_secret,
            config,
            rand_length: 6,
            timeout: 60,
            timeskip: 20,
        }
    }
    fn parse_state(&self, state: &str) -> Result<(String, String), String> {
        if state.is_empty() {
            return Err("state miss".to_string());
        }
        let state_rand = state.chars().take(self.rand_length).collect::<String>();
        let statek = state
            .chars()
            .skip(self.rand_length)
            .take(6)
            .collect::<String>();
        Ok((statek, state_rand))
    }
    // 微信扫码登陆完成后,进行登录数据回写
    pub async fn state_callback(
        &self,
        webuser: &WebUser,
        user_auth: &WechatCallbackParam,
    ) -> JsonResult<JsonData> {
        let (statek, _) = self
            .parse_state(&user_auth.state)
            .map_err(JsonData::message_error)?;
        let login_key = login_data_key(&statek);
        let mut redis = webuser.redis.get().await?;
        let login_data = serde_json::to_string(&user_auth)?;
        redis
            .set_ex(&login_key, login_data, self.timeout)
            .await
            .map_err(|e| JsonData::message_error(e.to_string()))?;
        Ok(JsonData::message("ok"))
    }
    // pc定时从服务器获取登陆数据
    pub async fn state_check(
        &self,
        webuser: &WebUser,
        state: &str,
    ) -> JsonResult<(bool, Option<WechatCallbackParam>)> {
        let state_ukey = &state.chars().take(6).collect::<String>();
        let state_key = state_key(state_ukey);
        let mut redis = webuser.redis.get().await?;
        let data_opt: Option<String> = redis
            .get(state_key.as_str())
            .await
            .map_err(|e| JsonData::message_error(e.to_string()))?;
        let data = data_opt.unwrap_or_default();
        let ttl: usize = redis
            .ttl(state_key.as_str())
            .await
            .map_err(|e| JsonData::message_error(e.to_string()))?;
        let reload = data.is_empty() || self.timeout < ttl + self.timeskip;
        if !data.is_empty() {
            let login_key = login_data_key(state_ukey);
            let data_opt: Option<String> = redis
                .get(login_key.as_str())
                .await
                .map_err(|e| JsonData::message_error(e.to_string()))?;
            let data = data_opt.unwrap_or_default();
            if data.is_empty() {
                return Ok((false, None));
            }
            return Ok((
                false,
                Some(serde_json::from_str::<WechatCallbackParam>(&data)?),
            ));
        };
        Ok((reload, None))
    }
}

#[async_trait]
impl OauthLogin<WechatLoginParam, WechatCallbackParam, WechatExternalData> for WechatLogin {
    async fn load_config(webuser: &WebUser, key: &str) -> Result<Self, String>
    where
        Self: std::marker::Sized,
    {
        let config = &webuser.app_core.config;
        let wx_config = config
            .get_table(&format!("oauth_{}", key))
            .map_err(|e| format!("wechat config err:{}", e))?;
        let app_id = wx_config
            .get("app_id")
            .ok_or_else(|| format!("[{}]config not wechat app id", key))?
            .to_owned()
            .into_string()
            .unwrap_or_default();
        let app_secret = wx_config
            .get("app_secret")
            .ok_or_else(|| format!("[{}]config not wechat app secret", key))?
            .to_owned()
            .into_string()
            .unwrap_or_default();
        Ok(WechatLogin::new(app_id, app_secret, key.to_owned()))
    }
    async fn login_url(
        &self,
        webuser: &WebUser,
        param: &WechatLoginParam,
    ) -> Result<String, String> {
        let state_ukey = &if param.state.is_empty() {
            state_rand(self.rand_length)
        } else {
            param.state.chars().take(6).collect::<String>()
        };
        if state_ukey.len() < 5 {
            return Err("state length can't <5".to_string());
        }
        let state_rand = state_rand(self.rand_length);
        let state_key = state_key(state_ukey);
        let mut redis = webuser.redis.get().await.map_err(|e| e.to_string())?;
        redis
            .set_ex(state_key.as_str(), state_rand.clone(), self.timeout)
            .await
            .map_err(|e| e.to_string())?;
        let c = WechatCpClient::<SimpleStorage>::new(
            self.app_id.to_owned(),
            self.app_secret.to_owned(),
        );
        let url = c.oauth2().build_authorization_url(
            &param.callback_url,
            "snsapi_userinfo",
            Some(format!("{}{}", state_rand, state_ukey).as_str()),
        );
        Ok(url)
    }
    async fn login_callback(
        &self,
        webuser: &WebUser,
        param: &WechatCallbackParam,
    ) -> Result<(OauthLoginData, WechatExternalData), String> {
        let (statek, state_rand) = self.parse_state(&param.state)?;
        let state_key = state_key(&statek);
        let mut redis = webuser.redis.get().await.map_err(|e| e.to_string())?;
        let save_state_opt: Option<String> = redis
            .get(state_key.as_str())
            .await
            .map_err(|e| e.to_string())?;
        let save_state_rand = save_state_opt.unwrap_or_default();
        if state_rand != save_state_rand {
            return Err("state timeout or wrong".to_string());
        }
        let c = WechatMpClient::<SimpleStorage>::new(
            self.app_id.to_owned(),
            self.app_secret.to_owned(),
        );
        let auth = c.oauth2();
        let resp = auth
            .oauth2_token(&param.code)
            .await
            .map_err(|e| e.to_string())?;
        let info = auth
            .oauth2_userinfo(&resp.access_token, &resp.openid)
            .await
            .map_err(|e| e.to_string())?;
        Ok((
            OauthLoginData {
                config_name: self.config.to_owned(),
                external_type: OAUTH_TYPE_WECHAT.to_string(),
                external_name: info.unionid.unwrap_or_else(|| resp.openid.clone()),
                external_id: resp.openid,
                external_gender: Some(info.sex.to_string()),
                external_link: None,
                external_pic: Some(info.headimgurl),
                external_nikename: info.nickname,
                token_data: resp.access_token,
                token_timeout: resp.expires_in as u64,
            },
            WechatExternalData {},
        ))
    }
}
