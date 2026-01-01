use crate::{
    now_time, valid_key, ValidError, ValidIp, ValidParam, ValidParamCheck, ValidPattern,
    ValidStrlen,
};

#[derive(Clone)]
pub struct RequestEnv {
    pub request_time: u64,
    pub request_lang: Option<String>,
    pub request_ip: Option<String>,
    pub request_id: Option<String>,
    pub request_user_agent: Option<String>,
    pub device_id: Option<String>,
}

impl RequestEnv {
    pub fn new(
        request_lang: Option<&str>,
        request_ip: Option<&str>,
        request_id: Option<&str>,
        request_user_agent: Option<&str>,
        device_id: Option<&str>,
    ) -> Result<Self, ValidError> {
        if request_lang.is_some()
            || request_ip.is_some()
            || request_id.is_some()
            || request_user_agent.is_some()
            || device_id.is_some()
        {
            let mut valid_param = ValidParam::default();
            if let Some(tmp) = request_lang {
                valid_param.add(
                    valid_key!("request_lang"),
                    &tmp,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::NotFormat)
                        .add_rule(ValidStrlen::range(5, 12)),
                );
            }
            if let Some(tmp) = request_ip {
                valid_param.add(
                    valid_key!("request_ip"),
                    &tmp,
                    &ValidParamCheck::default().add_rule(ValidIp::default()),
                );
            }
            if let Some(tmp) = request_id {
                valid_param.add(
                    valid_key!("request_id"),
                    &tmp,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::Ident)
                        .add_rule(ValidStrlen::range(8, 64)),
                );
            }
            if let Some(tmp) = request_user_agent {
                valid_param.add(
                    valid_key!("request_user_agent"),
                    &tmp,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::NotFormat)
                        .add_rule(ValidStrlen::range(1, 254)),
                );
            }
            if let Some(tmp) = device_id {
                valid_param.add(
                    valid_key!("device_id"),
                    &tmp,
                    &ValidParamCheck::default()
                        .add_rule(ValidPattern::NotFormat)
                        .add_rule(ValidStrlen::range(1, 64)),
                );
            }
            valid_param.check()?;
        }
        Ok(Self {
            request_lang: request_lang.map(|e| e.to_string()),
            request_time: now_time().unwrap_or_default(),
            request_ip: request_ip.map(|e| e.to_string()),
            request_id: request_id.map(|e| e.to_string()),
            device_id: device_id.map(|e| e.to_string()),
            request_user_agent: request_user_agent.map(|e| e.to_string()),
        })
    }
}
