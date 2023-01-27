use async_trait::async_trait;
use serde::Serialize;

use crate::dao::user::WebUser;

pub struct OauthLoginData {
    pub config_name: String,
    pub external_type: String,
    pub external_id: String,
    pub external_name: String,
    pub external_gender: Option<String>,
    pub external_link: Option<String>,
    pub external_pic: Option<String>,
    pub external_nikename: String,
    pub token_data: String,
    pub token_timeout: u64,
}

pub trait OauthLoginParam {}
pub trait OauthCallbackParam {}

#[async_trait]
pub trait OauthLogin<
    L: OauthLoginParam + Send + Sync,
    T: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>
{
    async fn load_config(webuser: &WebUser, key: &str) -> Result<Self, String>
    where
        Self: std::marker::Sized;
    async fn login_url(&self, webuser: &WebUser, param: &L) -> Result<String, String>;
    async fn login_callback(
        &self,
        webuser: &WebUser,
        param: &T,
    ) -> Result<(OauthLoginData, D), String>;
}
