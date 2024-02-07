use async_trait::async_trait;

use lsys_core::{fluent_message, rand_str, IntoFluentMessage, RandType};
use lsys_web::{
    dao::{user::WebUser, RequestDao},
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginData, OauthLoginParam},
    JsonData, JsonResult,
};
// use rand::seq::SliceRandom;

use redis::AsyncCommands;
use serde::{Deserialize, Serialize};

use super::{WeChatConfig, WeChatLib};

pub const OAUTH_TYPE_WECHAT: &str = "wechat";

// fn state_rand(len: usize) -> String {
//     const BASE_STR: &str = "0123456789";
//     let mut rng = &mut rand::thread_rng();
//     String::from_utf8(
//         BASE_STR
//             .as_bytes()
//             .choose_multiple(&mut rng, len)
//             .cloned()
//             .collect(),
//     )
//     .unwrap_or_default()
// }
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
    config: String,
    rand_length: usize,
    timeout: usize,  //state保存时间
    timeskip: usize, //剩余多少秒时加载下一个二维码
    lib: WeChatLib,
}

impl WechatLogin {
    pub fn new(app_id: String, app_secret: String, config: String) -> Self {
        Self {
            lib: WeChatLib::new(app_id, app_secret, None),
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
        req_dao: &RequestDao,
        user_auth: &WechatCallbackParam,
    ) -> JsonResult<JsonData> {
        let (statek, _) = self.parse_state(&user_auth.state).map_err(|e| {
            req_dao.fluent_json_data(fluent_message!("wechat-parse-state-error", e))
        })?;
        let login_key = login_data_key(&statek);
        let mut redis = req_dao
            .web_dao
            .user
            .redis
            .get()
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        let login_data =
            serde_json::to_string(&user_auth).map_err(|e| req_dao.fluent_json_data(e))?;
        redis
            .set_ex(&login_key, login_data, self.timeout)
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        Ok(JsonData::default())
    }
    // pc定时从服务器获取登陆数据
    pub async fn state_check(
        &self,
        req_dao: &RequestDao,
        state: &str,
    ) -> JsonResult<(bool, Option<WechatCallbackParam>)> {
        let state_ukey = &state.chars().take(6).collect::<String>();
        let state_key = state_key(state_ukey);
        let mut redis = req_dao
            .web_dao
            .user
            .redis
            .get()
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        let data_opt: Option<String> = redis
            .get(state_key.as_str())
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        let data = data_opt.unwrap_or_default();
        let ttl: usize = redis
            .ttl(state_key.as_str())
            .await
            .map_err(|e| req_dao.fluent_json_data(e))?;
        let reload = data.is_empty() || self.timeout < ttl + self.timeskip;
        if !data.is_empty() {
            let login_key = login_data_key(state_ukey);
            let data_opt: Option<String> = redis
                .get(login_key.as_str())
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?;
            let data = data_opt.unwrap_or_default();
            if data.is_empty() {
                return Ok((false, None));
            }
            return Ok((
                false,
                Some(
                    serde_json::from_str::<WechatCallbackParam>(&data)
                        .map_err(|e| req_dao.fluent_json_data(e))?,
                ),
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
        let config = webuser
            .setting
            .single
            .load::<WeChatConfig>(&None)
            .await
            .map_err(|e| {
                format!(
                    "load wechat error:{}",
                    e.to_fluent_message().default_format()
                )
            })?;
        Ok(WechatLogin::new(
            config.app_id.to_owned(),
            config.app_secret.to_owned(),
            key.to_owned(),
        ))
    }
    async fn login_url(
        &self,
        webuser: &WebUser,
        param: &WechatLoginParam,
    ) -> Result<String, String> {
        let state_ukey = &if param.state.is_empty() {
            rand_str(RandType::Number, self.rand_length)
        } else {
            param.state.chars().take(6).collect::<String>()
        };
        if state_ukey.len() < 5 {
            return Err("state length can't <5".to_string());
        }
        let state_rand = rand_str(RandType::Number, self.rand_length);
        let state_key = state_key(state_ukey);
        let mut redis = webuser.redis.get().await.map_err(|e| e.to_string())?;
        redis
            .set_ex(state_key.as_str(), state_rand.clone(), self.timeout)
            .await
            .map_err(|e| e.to_string())?;
        let url = self.lib.build_authorization_url(
            &param.callback_url,
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
        let resp = self.lib.oauth2_token(&param.code).await?;
        let info = self
            .lib
            .oauth2_userinfo(&resp.access_token, &resp.openid)
            .await?;
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
