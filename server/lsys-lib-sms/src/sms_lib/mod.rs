use base64::{
    alphabet,
    engine::{self, general_purpose},
};
use rand::seq::SliceRandom;
use reqwest::Response;
use reqwest::StatusCode;

use std::{
    collections::HashMap,
    time::{SystemTime, SystemTimeError},
};
use tracing::debug;
const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::PAD);

#[derive(Debug)]
pub enum SendError {
    Next(String),
    Finish(String),
}

#[derive(Debug, Clone)]
pub enum SendStatus {
    Progress,     //发送中
    Completed,    //发送成功或完成接收
    Failed(bool), //发送失败(可重试)
}

#[derive(Debug)]
pub struct SendResultItem {
    pub mobile: String,
    pub status: SendStatus,
    pub message: String,
    pub send_id: String,
}

pub type BranchSendResult = Result<Vec<SendResultItem>, SendError>;

#[derive(Debug)]
pub struct SendDetailItem {
    pub send_id: String,
    pub status: SendNotifyStatus,
    pub message: String,
    pub code: String,
    pub receive_time: Option<u64>,
    pub send_time: Option<u64>,
    pub mobile: Option<String>,
}

pub type BranchSendDetailResult = Result<Vec<SendDetailItem>, String>;

#[derive(Debug, Clone)]
pub enum SendNotifyStatus {
    Progress,  //发送中
    Completed, //完成接收或发送成功
    Failed,    //发送失败
}

#[derive(Debug)]
pub enum SendNotifyError {
    Msg(String),
    Sign(String),
    Ignore,
}

#[derive(Debug, Clone)]
pub struct SendNotifyItem {
    pub status: SendNotifyStatus,
    pub message: String,
    pub code: String,
    pub receive_time: Option<u64>,
    pub send_time: Option<u64>,
    pub send_id: String,
    pub mobile: Option<String>,
}

pub type BranchSendNotifyResult = Result<Vec<SendNotifyItem>, SendNotifyError>;

pub(crate) fn rand_str(len: usize) -> String {
    let base_str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = &mut rand::thread_rng();
    String::from_utf8(
        base_str
            .as_bytes()
            .choose_multiple(&mut rng, len)
            .cloned()
            .collect(),
    )
    .unwrap_or_default()
}

pub(crate) fn now_time() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub(crate) fn phone_numbers_check<'t>(
    phone_numbers: &'t [&'t str],
) -> Result<Vec<&'t str>, SendError> {
    let out = phone_numbers
        .iter()
        .map(|e| e.trim())
        .filter(|e| !e.is_empty())
        .collect::<Vec<_>>();
    if out.is_empty() {
        return Err(SendError::Finish("not find any mobile".to_string()));
    }
    Ok(out)
}

pub(crate) async fn response_check(
    result: Response,
    is_json: bool,
) -> Result<(StatusCode, String), String> {
    let status = result.status();
    let data = result
        .bytes()
        .await
        .map_err(|e| format!("request read body fail:{}", e))?;
    let res = unsafe { String::from_utf8_unchecked(data.to_vec()) };
    //println!("{}", res);
    debug!("sms response succ: {}", &res);
    if is_json && !gjson::valid(&res) {
        return Err(format!("body not json :{}", res));
    }
    Ok((status, res))
}

pub(crate) fn response_msg(result: &str, paths: &[&str]) -> String {
    for path in paths {
        let msg = gjson::get(result, path).to_string();
        if !msg.is_empty() {
            return format!("api fail,msg:{}", msg);
        }
    }
    format!("api fail,data:{}", result)
}

pub fn template_map_to_arr(template_var: &str, template_map: &str) -> Option<Vec<String>> {
    if let Ok(tmp) = serde_json::from_str::<HashMap<String, String>>(template_var) {
        let map_data = template_map.split(',');
        let mut set_data = vec![];
        if !tmp.is_empty() {
            for sp in map_data {
                if let Some(tv) = tmp.get(sp) {
                    set_data.push(tv.to_owned())
                }
            }
        }
        if !set_data.is_empty() {
            return Some(set_data);
        }
    }
    None
}
#[cfg(feature = "aliyun")]
mod sender_aliyun;
#[cfg(feature = "aliyun")]
pub use sender_aliyun::*;
#[cfg(feature = "huawei")]
mod sender_huawei;
#[cfg(feature = "huawei")]
pub use sender_huawei::*;

#[cfg(feature = "tencent")]
mod sender_tencent;
#[cfg(feature = "tencent")]
pub use sender_tencent::*;

#[cfg(feature = "jdcloud")]
mod sender_jdcloud;
#[cfg(feature = "jdcloud")]
pub use sender_jdcloud::*;

#[cfg(feature = "netease")]
mod sender_netease;
#[cfg(feature = "netease")]
pub use sender_netease::*;

#[cfg(feature = "cloopen")]
mod sender_cloopen;
#[cfg(feature = "cloopen")]
pub use sender_cloopen::*;
