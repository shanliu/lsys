use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Client;
use reqwest::Method;

use serde_json::json;

use sha2::{Digest, Sha256};
use tracing::debug;

use super::{BranchSendResult, SendError};
use crate::SendNotifyStatus;
use crate::{
    now_time, rand_str, response_check, response_msg, sms_lib::phone_numbers_check,
    BranchSendDetailResult, SendDetailItem, SendResultItem, SendStatus,
};

pub struct JdSms {}

impl JdSms {
    pub async fn send_detail(
        client: Client,
        internal: bool,
        region: &str,
        access_key: &str,
        access_secret: &str,
        sendid: &str,
        mobile: Option<Vec<String>>,
    ) -> BranchSendDetailResult {
        let mut reqjson = json!({
            "sequenceNumber": sendid,
        });
        if let Some(phone) = mobile {
            reqjson["phoneList"] = json!(phone);
        }
        let res = Self::build_request(
            client,
            internal,
            Method::POST,
            format!("/v1/regions/{}/statusReport", region).as_str(),
            "",
            region,
            "sms",
            &reqjson.to_string(),
            access_key,
            access_secret,
        )
        .await?;
        // println!("{}", res);
        //  let code = gjson::get(&res, "result.code");
        let status = gjson::get(&res, "result.status");
        if status.bool() {
            let items = gjson::get(&res, "result.data");
            let mut out = Vec::with_capacity(items.array().len());
            for tmp in items.array() {
                // phoneNum	String	手机号
                // sequenceNumber	String	发送短信的序列号
                // sendTime	String	短信发送时间（yyyy-MM-dd HH:mm:ss)
                // reportTime	String	接收到回执的时间（yyyy-MM-dd HH:mm:ss)
                // status	Integer	发送状态
                // code	String	错误码
                // splitNum	Integer	长短信拆分序号（短短信直接返回1)

                let send_time = chrono::NaiveDateTime::parse_from_str(
                    tmp.get("sendTime").str(),
                    "%Y-%m-%d %H:%M:%S",
                )
                .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
                .ok();
                let receive_time = chrono::NaiveDateTime::parse_from_str(
                    tmp.get("reportTime").str(),
                    "%Y-%m-%d %H:%M:%S",
                )
                .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
                .ok();
                out.push(SendDetailItem {
                    send_id: tmp.get("sequenceNumber").to_string(),
                    status: if tmp.get("status").i8() == 2 || tmp.get("status").i8() == 3 {
                        SendNotifyStatus::Failed
                    } else if tmp.get("status").i8() == 1 {
                        SendNotifyStatus::Completed
                    } else {
                        SendNotifyStatus::Progress
                    },
                    message: gjson::get(&res, "result.message").to_string(),
                    send_time,
                    receive_time,
                    code: tmp.get("code").to_string(),
                    mobile: Some(tmp.get("phoneNum").to_string()),
                });
            }
            return Ok(out);
        }
        Err(response_msg(&res, &["result.message", "error.message"]))
    }

    pub fn branch_limit() -> u16 {
        100
    }
    //执行短信发送
    #[allow(clippy::too_many_arguments)]
    pub async fn branch_send(
        client: Client,
        internal: bool,
        region: &str,
        access_key: &str,
        access_secret: &str,
        sign_name: &str,
        template_id: &str,
        template_arr: Option<Vec<String>>,
        phone_numbers: &[&str],
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;

        //         templateId	String	True		模板Id
        // signId	String	True		签名Id
        // phoneList	String[ ]	True		群发的国内电话号码,群发时一次最多不要超过100个手机号
        // params	String[ ]	False		短信模板变量对应的数据值,Array格式

        let mut reqjson = json!({
            "templateId" :template_id,
            "signId" : sign_name,
            "phoneList": phone_numbers,
        });
        if let Some(data_var) = template_arr {
            reqjson["params"] = data_var.into();
        }
        let res = Self::build_request(
            client,
            internal,
            Method::POST,
            format!("/v1/regions/{}/batchSend", region).as_str(),
            "",
            region,
            "sms",
            &reqjson.to_string(),
            access_key,
            access_secret,
        )
        .await
        .map_err(SendError::Next)?;

        // println!("{}", res);

        let code = gjson::get(&res, "result.code");
        let status = gjson::get(&res, "result.status");
        if code.i64() == 200 && status.bool() {
            return Ok(phone_numbers
                .iter()
                .map(|e| SendResultItem {
                    mobile: e.to_string(),
                    status: SendStatus::Progress,
                    message: gjson::get(&res, "result.message").to_string(),
                    send_id: gjson::get(&res, "result.data.sequenceNumber").to_string(),
                })
                .collect());
        }
        Err(SendError::Next(response_msg(
            &res,
            &["result.message", "error.message"],
        )))
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn build_request(
        client: Client,
        internal: bool,
        method: Method,
        uri: &str,
        query: &str,
        region: &str,
        service: &str,
        req_json: &str,
        secret_id: &str,
        secret_key: &str,
    ) -> Result<String, String> {
        let now_time = now_time().unwrap_or_default();

        let datetime = DateTime::from_timestamp(now_time as i64, 0).unwrap_or_default();

        let datetime_str = datetime.format("%Y%m%dT%H%M%SZ").to_string();
        let rand_s = rand_str(32);

        // content-type: application/json \n
        // host: vm.jdcloud-api.com \n
        // my-header1:a b c\n
        // my-header2:"a b c"\n
        // x-jdcloud-date: 20180404T061302Z\n
        // x-jdcloud-nonce:ed558a3b-9808-4edb-8597-187bda63a4f2\n
        //https://sms.jdcloud-api.com/v1/regions/{regionId}/statusReport
        let host = if internal {
            format!(
                "https://{}.internal.{}.jdcloud-api.com{}",
                service, region, uri
            )
        } else {
            format!("https://{}.jdcloud-api.com{}", service, uri)
        };

        let mut headers = HeaderMap::new();
        // if let Ok(value) = HeaderValue::from_str(host.as_str()) {
        //     headers.insert("Host", value);
        // }
        if let Ok(value) = HeaderValue::from_str("application/json") {
            headers.insert("Content-type", value);
        }
        if let Ok(value) = HeaderValue::from_str("JDCLOUD2-HMAC-SHA256") {
            headers.insert("x-jdcloud-algorithm", value);
        }

        if let Ok(value) = HeaderValue::from_str(&datetime_str) {
            headers.insert("x-jdcloud-date", value);
        }
        if let Ok(value) = HeaderValue::from_str(&rand_s) {
            headers.insert("x-jdcloud-nonce", value);
        }

        let reqjson = req_json.to_string();

        let mut hasher = Sha256::new();
        hasher.update(&reqjson);
        let result = hasher.finalize();

        let sign_header_arr = &["x-jdcloud-date", "x-jdcloud-nonce"];
        let sign_header = sign_header_arr.join(";");
        let mut my_header = Vec::with_capacity(sign_header_arr.len());
        for tmp in sign_header_arr {
            if let Some(tval) = headers.get(*tmp) {
                my_header.push(format!("{}:{}", tmp, tval.to_str().unwrap_or_default()))
            } else {
                my_header.push(format!("{}:{}", tmp, ""))
            }
        }

        let sign = format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            method.as_str(),
            uri,
            query,
            my_header.join("\n"),
            sign_header,
            format!("{:x}", result).to_lowercase()
        );

        //println!("BODY:\n{}", sign);

        let now_date = datetime.format("%Y%m%d").to_string();
        debug!("post json:{}  header:{}", reqjson, sign);

        let mut hasher1 = Sha256::new();
        hasher1.update(&sign);
        let result = hasher1.finalize();

        let string_to_sign = format!(
            "JDCLOUD2-HMAC-SHA256\n{}\n{}/{}/{}/jdcloud2_request\n{}",
            datetime_str,
            now_date,
            region,
            service,
            format!("{:x}", result).to_lowercase()
        );

        // println!("SIGN:\n{}", string_to_sign);

        debug!("sign body:{}", string_to_sign);

        let mut mac =
            Hmac::<sha2::Sha256>::new_from_slice(format!("JDCLOUD2{}", secret_key).as_bytes())
                .map_err(|e| format!("use data key on sha256 fail:{}", e))?;
        mac.update(now_date.as_bytes());
        let result = mac.finalize();

        let mut mac1 = Hmac::<sha2::Sha256>::new_from_slice(&result.into_bytes())
            .map_err(|e| format!("use  on sha256 fail:{}", e))?;
        mac1.update(region.as_bytes());
        let result = mac1.finalize();

        let mut mac2 = Hmac::<sha2::Sha256>::new_from_slice(&result.into_bytes())
            .map_err(|e| format!("use  on sha256 fail:{}", e))?;
        mac2.update(service.as_bytes());
        let result = mac2.finalize();

        let mut mac3 = Hmac::<sha2::Sha256>::new_from_slice(&result.into_bytes())
            .map_err(|e| format!("use tc3_request on sha256 fail:{}", e))?;
        mac3.update(b"jdcloud2_request");
        let secret_signing = mac3.finalize();

        let mut tmp3 = Hmac::<sha2::Sha256>::new_from_slice(&secret_signing.into_bytes())
            .map_err(|e| format!("use tc3_request on sha256 fail:{}", e))?;
        tmp3.update(string_to_sign.as_bytes());
        let signature = tmp3.finalize();

        let data_sign = hex::encode(signature.into_bytes());

        //JDCLOUD2-HMAC-SHA256 Credential={Access Key}/{Date}/{Region}/{Service}/jdcloud2_request, SignedHeaders={SignedHeaders}, Signature={signResult}
        let authdata =format!(
                     "JDCLOUD2-HMAC-SHA256 Credential={}/{}/{}/{}/jdcloud2_request, SignedHeaders={}, Signature={}",
                     secret_id,
                     datetime.format("%Y%m%d"),
                     region,
                     service,
                     sign_header,
                     data_sign
                 );

        debug!("jd authorization: {}", authdata);

        if let Ok(value) = HeaderValue::from_str(&authdata) {
            headers.insert("authorization", value);
        }

        let request = client.request(method, host).headers(headers).body(reqjson);
        let result = request
            .send()
            .await
            .map_err(|e| format!("request send fail:{}", e))?;
        let (_, res) = response_check(result, true).await?;
        Ok(res)
    }
}
