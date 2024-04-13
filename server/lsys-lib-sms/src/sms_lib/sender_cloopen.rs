use base64::Engine;
use chrono::{DateTime, Utc};

use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::Method;
use reqwest::StatusCode;
use serde_json::json;

use tracing::warn;

use super::{BranchSendResult, SendError};
use crate::BranchSendNotifyResult;
use crate::SendNotifyError;
use crate::SendNotifyItem;
use crate::SendNotifyStatus;
use crate::{
    now_time, response_check, response_msg, sms_lib::phone_numbers_check, BranchSendDetailResult,
    SendDetailItem, SendResultItem, SendStatus,
};

pub struct CloOpenSms {}

impl CloOpenSms {
    pub fn send_notify_output(res: &Result<(), String>) -> (StatusCode, String) {
        match res {
            Ok(_) => (StatusCode::OK, "".to_owned()),
            Err(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_owned()),
        }
    }
    //send_detail 跟  send_notify_parse 只有一个
    //post json
    pub fn send_notify_parse(notify_data: &str) -> BranchSendNotifyResult {
        // {
        //     "Request": {
        //     "action": "SMSArrived",
        //     "smsType": "1",
        //     "apiVersion": "2013-12-26",
        //     "content": "4121908f3d1b4edb9210f0eb4742f62c",
        //     "fromNum": "13912345678",
        //     "dateSent": "20130923010000",
        //     "deliverCode": "DELIVRD",
        //     "recvTime": "20130923010010",
        //     "status": "0",
        //     "reqId": "123",
        //     "smsCount": "2",
        //     "spCode": "10690876"
        //     }
        //     }
        if gjson::get(notify_data, "Request.smsType")
            .to_string()
            .as_str()
            != "1"
        {
            return Err(SendNotifyError::Ignore);
        }
        let tmp = gjson::get(notify_data, "Request");

        let receive_time =
            chrono::NaiveDateTime::parse_from_str(&tmp.get("recvTime").to_string(), "%Y%m%d%H%M%S")
                .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
                .ok();
        let send_time =
            chrono::NaiveDateTime::parse_from_str(&tmp.get("dateSent").to_string(), "%Y%m%d%H%M%S")
                .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
                .ok();

        Ok(vec![SendNotifyItem {
            send_id: tmp.get("content").to_string(),
            status: if send_time.unwrap_or_default() > 0 {
                if tmp.get("status").i8() == 0 {
                    SendNotifyStatus::Completed
                } else {
                    SendNotifyStatus::Failed
                }
            } else {
                SendNotifyStatus::Progress
            },
            message: tmp.get("deliverCode").to_string(),
            send_time,
            receive_time,
            code: tmp.get("deliverCode").to_string(),
            mobile: Some(tmp.get("fromNum").to_string()),
        }])
    }
    /// 构建签名字符串
    fn sig(account_sid: &str, account_token: &str, time: u64) -> String {
        let datetime = DateTime::from_timestamp(time as i64, 0).unwrap_or_default();
        let datetime_str = datetime.format("%Y%m%d%H%M%S").to_string();
        let sig_str = format!("{}{}{}", account_sid, account_token, datetime_str);
        let result = md5::compute(sig_str);
        format!("{:x}", result).to_uppercase()
    }
    /// 构建验证字符串
    fn auth(account_sid: &str, time: u64) -> String {
        let datetime = DateTime::from_timestamp(time as i64, 0).unwrap_or_default();
        let datetime_str = datetime.format("%Y%m%d%H%M%S").to_string();
        let auth_str = format!("{}:{}", account_sid, datetime_str);
        base64::engine::general_purpose::STANDARD.encode(auth_str)
    }

    pub async fn send_detail(
        client: Client,
        account_sid: &str,
        account_token: &str,
        app_id: &str,
    ) -> BranchSendDetailResult {
        let ntime = now_time().unwrap_or_default();
        let sig = Self::sig(account_sid, account_token, ntime);
        let auth = Self::auth(account_sid, ntime);
        // appId	String	必选	应用Id
        // 	String	可选	0:上行短信数据 1:短信状态报告 缺省1
        // 	String	可选	查询状态的数量。最大500，缺省100
        let reqjson = json!({
            "smsType": "1",
            "count": "500",
            "appId": app_id,
        });

        let mut headers = HeaderMap::new();

        if let Ok(value) = HeaderValue::from_str("application/json;charset=utf-8") {
            headers.insert("Content-Type", value);
        }
        if let Ok(value) = HeaderValue::from_str(auth.as_str()) {
            headers.insert("Authorization", value);
        }

        let request = client
            .request(
                Method::POST,
                format!(
                    "https://app.cloopen.com:8883/2013-12-26/Accounts/{}/SMS/GetArrived?sig={}",
                    account_sid, sig
                ),
            )
            .headers(headers)
            .body(reqjson.to_string());
        let result = request.send().await.map_err(|e| e.to_string())?;
        let (status, res) = response_check(result, true).await?;
        // println!("{}", res);
        if status != StatusCode::OK {
            warn!("sms response fail: {}", &res);
            return Err(format!("http bad:{}", res));
        }
        let code = gjson::get(&res, "statusCode").to_string();
        if code.as_str() == "000000" {
            let items = gjson::get(&res, "reports");
            let mut out = Vec::with_capacity(items.array().len());
            for tmp in items.array() {
                let receive_time = chrono::NaiveDateTime::parse_from_str(
                    &tmp.get("recvTime").to_string(),
                    "%Y%m%d%H%M%S",
                )
                .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
                .ok();
                let send_time = chrono::NaiveDateTime::parse_from_str(
                    &tmp.get("dateSent").to_string(),
                    "%Y%m%d%H%M%S",
                )
                .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
                .ok();
                out.push(SendDetailItem {
                    send_id: tmp.get("content").to_string(),
                    status: if send_time.unwrap_or_default() > 0 {
                        if tmp.get("status").i8() == 0 {
                            SendNotifyStatus::Completed
                        } else {
                            SendNotifyStatus::Failed
                        }
                    } else {
                        SendNotifyStatus::Progress
                    },
                    message: match tmp.get("status").i8() {
                        0 => "OK".to_owned(),
                        _ => "send fail".to_owned(),
                    },
                    send_time,
                    receive_time,
                    code: tmp.get("status").to_string(),
                    mobile: Some(tmp.get("fromNum").to_string()),
                });
            }
            return Ok(out);
        }
        Err(response_msg(&res, &["statusMsg"]))
    }
    pub fn branch_limit() -> u16 {
        200
    }
    //执行短信发送
    pub async fn branch_send(
        client: Client,
        account_sid: &str,
        account_token: &str,
        app_id: &str,
        template_id: &str,
        template_arr: Option<Vec<String>>,
        phone_numbers: &[&str],
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;
        let ntime = 1700447467; //now_time().unwrap_or_default();

        let sig = Self::sig(account_sid, account_token, ntime);
        let auth = Self::auth(account_sid, ntime);
        let reqjson = json!({
            "to": phone_numbers.join(","),
            "templateId": template_id,
            "appId": app_id,
            "datas":template_arr
        });

        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str("application/json") {
            headers.insert("Accept", value);
        }
        if let Ok(value) = HeaderValue::from_str("application/json;charset=utf-8") {
            headers.insert("Content-Type", value);
        }
        if let Ok(value) = HeaderValue::from_str(auth.as_str()) {
            headers.insert("Authorization", value);
        }

        let url = format!(
            "https://app.cloopen.com:8883/2013-12-26/Accounts/{}/SMS/TemplateSMS?sig={}",
            account_sid, sig
        );

        // println!("{}-{}", url, auth);

        let request = client
            .request(Method::POST, url)
            .headers(headers)
            .body(reqjson.to_string());
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
        //  println!("{}", res);
        let code = gjson::get(&res, "statusCode").to_string();
        if code.as_str() == "000000" {
            // {"statusCode":"000000","templateSMS":{"dateCreated":"20130201155306","smsMessageSid":" ff8080813c373cab013c94b0f0512345"}}
            return Ok(phone_numbers
                .iter()
                .map(|e| SendResultItem {
                    mobile: e.to_string(),
                    status: SendStatus::Progress,
                    message: gjson::get(&res, "statusMsg").to_string(),
                    send_id: gjson::get(&res, "templateSMS.smsMessageSid").to_string(),
                })
                .collect());
        }
        Err(SendError::Next(response_msg(&res, &["statusMsg"])))
    }
}
