use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::Method;
use reqwest::StatusCode;
use ring::digest;
use serde_json::json;
use tracing::{debug, info, warn};

use super::{BranchSendDetailResult, BranchSendResult, SendDetailItem, SendResultItem, SendStatus};

use crate::SendNotifyStatus;
use crate::{
    sms_lib::{now_time, phone_numbers_check, rand_str, response_check, response_msg, SendError},
    BranchSendNotifyResult, SendNotifyError, SendNotifyItem,
};

pub struct NeteaseSms {}
impl NeteaseSms {
    pub fn send_notify_output(res: &Result<(), String>) -> String {
        match res {
            Ok(_) => {
                json!({
                  "code" : 200,
                  "msg" : "接收成功"
                })
            }
            Err(err) => {
                json!({
                  "code" : 500,
                  "msg" : err
                })
            }
        }
        .to_string()
    }
    //post json

    pub fn send_notify_parse(
        notify_data: &str,
        // app_secret
        // header -> MD5: xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx //根据请求中的request body计算出来的MD5值
        // header -> CurTime: 1440570500855    //当前UTC时间戳，从1970年1月1日0点0 分0 秒开始到现在的毫秒数(String)
        // header -> CheckSum: 001511b8435e0b28044ca50a78e8f983026c5e01
        sign_data: Option<(&str, &str, &str, &str)>,
    ) -> BranchSendNotifyResult {
        if let Some((app_secret, header_md5, header_curtime, header_checksum)) = sign_data {
            let result = md5::compute(notify_data.as_bytes());
            let hex_string = format!("{:x}", result);
            debug!("body md5:{}", hex_string);
            if hex_string.to_lowercase() != header_md5.to_lowercase() {
                info!("md5 not match:{}!={}", hex_string, header_md5);
                return Err(SendNotifyError::Sign(format!(
                    "body md5 not match on :{}",
                    hex_string
                )));
            }
            let data = format!("{}{}{}", app_secret, hex_string, header_curtime);
            let actual = digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, data.as_bytes());
            let sign = hex::encode(actual.as_ref());
            if sign.to_lowercase() != header_checksum.to_lowercase() {
                info!("md5 not match:{}!={}", hex_string, header_checksum);
                return Err(SendNotifyError::Sign(format!("sign bad on :{}", sign)));
            }
        }
        if gjson::get(notify_data, "eventType").to_string().as_str() != "11" {
            return Err(SendNotifyError::Ignore);
        }
        // {
        //     "mobile": "12345678945",
        //     "sendid": "1490",
        //     "result": "DELIVRD",
        //     "sendTime": "2017-06-02 14:40:45",
        //     "reportTime": "2017-06-06 10:40:30",
        //     "spliced": "1",
        //     "templateId": 1234
        //     },
        let items = gjson::get(notify_data, "objects");
        let mut out = Vec::with_capacity(items.array().len());
        for tmp in items.array() {
            let send_time = chrono::NaiveDateTime::parse_from_str(
                &tmp.get("sendTime").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
            .ok();
            let receive_time = chrono::NaiveDateTime::parse_from_str(
                &tmp.get("reportTime").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
            .ok();

            out.push(SendNotifyItem {
                status: if tmp.get("result").str() == "DELIVRD" {
                    SendNotifyStatus::Completed
                } else {
                    SendNotifyStatus::Failed
                },
                message: tmp.get("result").to_string(),

                send_time,
                receive_time,
                code: tmp.get("result").to_string(),
                send_id: tmp.get("sendid").to_string(),
                mobile: Some(tmp.get("mobile").to_string()),
            });
        }
        Ok(out)
    }
    /// 构建签名字符串
    fn signature(app_secret: &str) -> (String, String, String) {
        let nowtime = now_time().unwrap_or_default();
        let randstring = rand_str(32);
        let check_sum = {
            let tmp = format!("{}{}{}", app_secret, randstring, nowtime);
            let sha1 = digest::digest(&digest::SHA1_FOR_LEGACY_USE_ONLY, tmp.as_bytes());
            hex::encode(sha1)
        };
        //println!("{}", check_sum);
        (randstring, nowtime.to_string(), check_sum)
    }

    pub async fn send_detail(
        client: Client,
        app_key: &str,
        app_secret: &str,
        sendid: &str,
    ) -> BranchSendDetailResult {
        let (randstring, nowtime, check_sum) = Self::signature(app_secret);

        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(app_key) {
            headers.insert("AppKey", value);
        }
        if let Ok(value) = HeaderValue::from_str(&randstring) {
            headers.insert("Nonce", value);
        }
        if let Ok(value) = HeaderValue::from_str(&nowtime) {
            headers.insert("CurTime", value);
        }
        if let Ok(value) = HeaderValue::from_str(&check_sum) {
            headers.insert("CheckSum", value);
        }
        if let Ok(value) = HeaderValue::from_str("application/x-www-form-urlencoded;charset=utf-8")
        {
            headers.insert("Content-Type", value);
        }

        let form_data = vec![("sendid", sendid)];

        let request = client
            .request(
                Method::POST,
                "https://api.netease.im/sms/querystatus.action",
            )
            .headers(headers)
            .form(&form_data);
        let result = request.send().await.map_err(|e| e.to_string())?;
        let (status, res) = response_check(result, true).await?;
        if status != StatusCode::OK {
            warn!("sms response fail: {}", &res);
            return Err(format!("http bad:{}", res));
        }
        let code = gjson::get(&res, "code").to_string();
        if code.as_str() == "200" {
            let items = gjson::get(&res, "obj");
            let mut out = Vec::with_capacity(items.array().len());
            for tmp in items.array() {
                out.push(SendDetailItem {
                    send_id: sendid.to_owned(),
                    status: if tmp.get("status").i8() == 2 || tmp.get("status").i8() == 3 {
                        SendNotifyStatus::Failed
                    } else if tmp.get("status").i8() == 1 {
                        SendNotifyStatus::Completed
                    } else {
                        SendNotifyStatus::Progress
                    },
                    message: match tmp.get("status").i8() {
                        1 => "OK".to_owned(),
                        3 => "spam".to_owned(),
                        _ => "send fail".to_owned(),
                    },
                    send_time: Some(tmp.get("status").u64() / 1000),
                    receive_time: None,
                    code: code.to_string(),
                    mobile: Some(tmp.get("mobile").to_string()),
                });
            }
            return Ok(out);
        }
        Err(response_msg(&res, &["obj"]))
    }
    pub fn branch_limit() -> u16 {
        100
    }
    //执行短信发送
    pub async fn branch_send(
        client: Client,
        app_key: &str,
        app_secret: &str,
        template_id: &str,
        template_arr: Option<Vec<String>>,
        phone_numbers: &[&str],
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;

        let (randstring, nowtime, check_sum) = Self::signature(app_secret);

        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(app_key) {
            headers.insert("AppKey", value);
        }
        if let Ok(value) = HeaderValue::from_str(&randstring) {
            headers.insert("Nonce", value);
        }
        if let Ok(value) = HeaderValue::from_str(&nowtime) {
            headers.insert("CurTime", value);
        }
        if let Ok(value) = HeaderValue::from_str(&check_sum) {
            headers.insert("CheckSum", value);
        }
        if let Ok(value) = HeaderValue::from_str("application/x-www-form-urlencoded;charset=utf-8")
        {
            headers.insert("Content-Type", value);
        }

        let mut form_data = vec![];
        form_data.push(("templateid", template_id));
        let mobile = json!(phone_numbers).to_string();
        form_data.push(("mobiles", &mobile));
        let ptmp = if let Some(params) = template_arr {
            json!(params).to_string()
        } else {
            "".to_string()
        };
        if !ptmp.is_empty() {
            form_data.push(("params", ptmp.as_str()));
        }

        let request = client
            .request(
                Method::POST,
                "https://api.netease.im/sms/sendtemplate.action",
            )
            .headers(headers)
            .form(&form_data);
        let result = request
            .send()
            .await
            .map_err(|e| SendError::Next(format!("request send fail:{}", e)))?;
        let (status, res) = response_check(result, true)
            .await
            .map_err(SendError::Next)?;
        if status != StatusCode::OK {
            warn!("sms response fail: {}", &res);
            return Err(SendError::Next(format!("http bad:{}", res)));
        }

        let code = gjson::get(&res, "code").to_string();
        if code.as_str() == "200" {
            return Ok(phone_numbers
                .iter()
                .map(|e| SendResultItem {
                    mobile: e.to_string(),
                    status: SendStatus::Progress,
                    message: gjson::get(&res, "obj").to_string(),
                    send_id: gjson::get(&res, "msg").to_string(),
                })
                .collect());
        }
        if code.as_str() == "412" || code.as_str() == "601" {
            return Err(SendError::Finish(response_msg(&res, &["obj"])));
        }
        Err(SendError::Next(response_msg(&res, &["obj"])))
    }
}
