use chrono::{DateTime, Utc};

use hmac::{Hmac, Mac};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Method,
};
use reqwest::{Client, StatusCode};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::SendNotifyStatus;
use crate::{
    response_check, response_msg, sms_lib::phone_numbers_check, BranchSendDetailResult,
    BranchSendNotifyResult, SendDetailItem, SendNotifyItem, SendResultItem, SendStatus,
};
use tracing::{debug, warn};

use super::SendError;

use super::{now_time, BranchSendResult};

pub struct TenSms {}

impl TenSms {
    pub fn send_notify_output(res: &Result<(), String>) -> String {
        match res {
            Ok(_) => {
                json!({
                    "result": 0,
                    "errmsg": "OK"
                })
            }
            Err(err) => {
                json!({
                    "result" : 500,
                    "errmsg" : err
                })
            }
        }
        .to_string()
    }
    //post json
    pub fn send_notify_parse(notify_data: &str) -> BranchSendNotifyResult {
        // [
        //     {
        //         "user_receive_time": "2015-10-17 08:03:04",
        //         "nationcode": "86",
        //         "mobile": "13xxxxxxxxx",
        //         "report_status": "SUCCESS",
        //         "errmsg": "DELIVRD",
        //         "description": "用户短信送达成功",
        //         "sid": "xxxxxxx"
        //     }
        // ]
        let items = gjson::parse(notify_data);
        let mut out = Vec::with_capacity(items.array().len());
        for tmp in items.array() {
            let receive_time = chrono::NaiveDateTime::parse_from_str(
                &tmp.get("user_receive_time").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
            .ok();
            out.push(SendNotifyItem {
                status: if tmp.get("report_status").str() == "SUCCESS" {
                    SendNotifyStatus::Completed
                } else {
                    SendNotifyStatus::Failed
                },
                message: tmp.get("description").to_string(),
                send_time: None,
                receive_time,
                code: tmp.get("errmsg").to_string(),
                send_id: tmp.get("sid").to_string(),
                mobile: Some(tmp.get("mobile").to_string()),
            });
        }
        Ok(out)
    }

    pub async fn send_detail(
        client: Client,
        region: &str,
        secret_id: &str,
        secret_key: &str,
        sms_app_id: &str,
        mobile: &str,
        send_date: &str,
    ) -> BranchSendDetailResult {
        let start_time = chrono::NaiveDateTime::parse_from_str(
            &format!("{} 00:00:00", send_date),
            "%Y-%m-%d %H:%M:%S",
        )
        .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
        .unwrap_or(now_time().unwrap_or_default() - 3600 * 24);

        let end_time = chrono::NaiveDateTime::parse_from_str(
            &format!("{} 23:59:59", send_date),
            "%Y-%m-%d %H:%M:%S",
        )
        .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
        .unwrap_or(now_time().unwrap_or_default() - 3600 * 24);

        let reqjson = json!({
            "PhoneNumber": mobile,
            "SmsSdkAppId": sms_app_id,
            "BeginTime": start_time,
            "Offset": 0,
            "Limit": 100,
            "EndTime":end_time,
        });

        let res = Self::build_request(
            client,
            "sms.tencentcloudapi.com",
            Method::POST,
            "sms",
            region,
            "PullSmsSendStatusByPhoneNumber",
            &reqjson.to_string(),
            secret_id,
            secret_key,
        )
        .await?;
        let code = gjson::get(&res, "Response.PullSmsSendStatusSet");
        if code.exists() {
            let mut out = Vec::with_capacity(code.array().len());
            for tmp in code.array() {
                // {
                //     "Description": "DELIVRD",
                //     "CountryCode": "86",
                //     "SubscriberNumber": "15291996666",
                //     "ReportStatus": "SUCCESS",
                //     "PhoneNumber": "+8615291996666",
                //     "SerialNo": "14:19325917feb3914eb78b50d6182d7e452e",
                //     "UserReceiveTime": 1620734188
                // },
                out.push(SendDetailItem {
                    status: if tmp.get("ReportStatus").str() == "SUCCESS" {
                        SendNotifyStatus::Completed
                    } else if tmp.get("ReportStatus").str() == "FAIL" {
                        SendNotifyStatus::Failed
                    } else {
                        SendNotifyStatus::Progress
                    },
                    message: tmp.get("Description").to_string(),
                    send_time: None,
                    receive_time: Some(tmp.get("UserReceiveTime").u64()),
                    code: gjson::get(&res, "SmsSendDetailDTOs.0.ErrCode").to_string(),
                    send_id: tmp.get("SerialNo").to_string(),
                    mobile: Some(tmp.get("PhoneNumber").to_string()),
                });
            }
            return Ok(out);
        }
        Err(response_msg(&res, &["Response.Error.Message"]))
    }
    pub fn branch_limit() -> u16 {
        200
    }
    //执行短信发送
    #[allow(clippy::too_many_arguments)]
    pub async fn branch_send(
        client: Client,
        region: &str,
        secret_id: &str,
        secret_key: &str,
        sms_app_id: &str,
        sign_name: &str,
        template_id: &str,
        template_arr: Option<Vec<String>>,
        phone_numbers: &[&str],
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;
        let mut reqjson = json!({
            "PhoneNumberSet": phone_numbers,
            "SmsSdkAppId": sms_app_id,
            "SignName": sign_name,
            "TemplateId": template_id,
        });
        if let Some(data_var) = template_arr {
            reqjson["TemplateParamSet"] = data_var.into();
        }

        let res = Self::build_request(
            client,
            "sms.tencentcloudapi.com",
            Method::POST,
            "sms",
            region,
            "SendSms",
            &reqjson.to_string(),
            secret_id,
            secret_key,
        )
        .await
        .map_err(SendError::Next)?;
        let code = gjson::get(&res, "Response.SendStatusSet");
        if code.exists() {
            let mut out = Vec::with_capacity(code.array().len());
            for tmp in code.array() {
                out.push(SendResultItem {
                    mobile: tmp.get("PhoneNumber").to_string(),
                    status: if tmp.get("Code").str() == "Ok" {
                        SendStatus::Progress
                    } else {
                        SendStatus::Failed(true)
                    },
                    message: tmp.get("Message").to_string(),
                    send_id: tmp.get("SerialNo").to_string(),
                })
            }
            return Ok(out);
        }
        Err(SendError::Next(response_msg(
            &res,
            &["Response.Error.Message"],
        )))
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn build_request(
        client: Client,
        host: &str,
        method: Method,
        service: &str,
        region: &str,
        action: &str,
        req_json: &str,
        secret_id: &str,
        secret_key: &str,
    ) -> Result<String, String> {
        let now_time = now_time().unwrap_or_default();
        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(host) {
            headers.insert("Host", value);
        }
        if let Ok(value) = HeaderValue::from_str(action) {
            headers.insert("X-TC-Action", value);
        }
        if let Ok(value) = HeaderValue::from_str("2021-01-11") {
            headers.insert("X-TC-Version", value);
        }
        if let Ok(value) = HeaderValue::from_str(&now_time.to_string()) {
            headers.insert("X-TC-Timestamp", value);
        }
        if let Ok(value) = HeaderValue::from_str("zh-CN") {
            headers.insert("X-TC-Language", value);
        }
        if let Ok(value) = HeaderValue::from_str(region) {
            headers.insert("X-TC-Region", value);
        }
        if let Ok(value) = HeaderValue::from_str("application/json") {
            headers.insert("Content-Type", value);
        }
        let datetime = DateTime::from_timestamp(now_time as i64, 0).unwrap_or_default();

        let reqjson = req_json.to_string();

        let mut hasher = Sha256::new();
        hasher.update(&reqjson);
        let result = hasher.finalize();

        let sign = format!(
             "{}\n/\n\ncontent-type:application/json\nhost:{}\nx-tc-action:{}\n\ncontent-type;host;x-tc-action\n{}",
             method.as_str(),
             host,
             action.to_lowercase(),
             format!("{:x}", result).to_lowercase()
         );

        let now_date = datetime.format("%Y-%m-%d").to_string();
        debug!("tencent post json:{}  header:{}", reqjson, sign);

        let mut hasher1 = Sha256::new();
        hasher1.update(&sign);
        let result = hasher1.finalize();

        let string_to_sign = format!(
            "TC3-HMAC-SHA256\n{}\n{}/{}/tc3_request\n{}",
            now_time,
            now_date,
            service,
            format!("{:x}", result).to_lowercase()
        );

        debug!("sign body:{}", string_to_sign);

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(format!("TC3{}", secret_key).as_bytes())
            .map_err(|e| format!("use data key on sha256 fail:{}", e))?;
        mac.update(now_date.as_bytes());
        let result = mac.finalize().into_bytes();

        let mut mac1 = Hmac::<sha2::Sha256>::new_from_slice(&result)
            .map_err(|e| format!("use on sha256 fail:{}", e))?;
        mac1.update(service.as_bytes());
        let result = mac1.finalize();

        let mut mac2 = Hmac::<sha2::Sha256>::new_from_slice(&result.into_bytes())
            .map_err(|e| format!("use tc3_request on sha256 fail:{}", e))?;
        mac2.update(b"tc3_request");
        let secret_signing = mac2.finalize();

        let mut tmp3 = Hmac::<sha2::Sha256>::new_from_slice(&secret_signing.into_bytes())
            .map_err(|e| format!("use tc3_request on sha256 fail:{}", e))?;
        tmp3.update(string_to_sign.as_bytes());
        let signature = tmp3.finalize();

        let data_sign = hex::encode(signature.into_bytes());

        let authdata =format!(
                     "TC3-HMAC-SHA256 Credential={}/{}/{}/tc3_request, SignedHeaders=content-type;host;x-tc-action, Signature={}",
                     secret_id,
                     datetime.format("%Y-%m-%d"),
                     service,
                     data_sign
                 );

        // println!("{}\n{}", authdata, string_to_sign);

        if let Ok(value) = HeaderValue::from_str(&authdata) {
            headers.insert("Authorization", value);
        }

        let request = client
            .request(method, format!("https://{}/", host))
            .headers(headers)
            .body(reqjson);
        let result = request
            .send()
            .await
            .map_err(|e| format!("Tenyun request send fail:{}", e))?;
        let (status, res) = response_check(result, true).await?;
        if status != StatusCode::OK {
            warn!("response fail: {}", &res);
            return Err(format!("http bad:{}", res));
        }
        Ok(res)
    }
}
