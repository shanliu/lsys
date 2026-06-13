use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::WebDao;
use lsys_access::dao::AccessSession;
use lsys_core::fluent_message;
use lsys_mfa::dao::MfaSubject;
use rand::seq::IndexedRandom;
use serde::Deserialize;
use serde_json::json;

/// 生成用户绑定MFA设备的二维码数据信息
/// 返回二维码URL、Secret密钥等信息供MFA设备扫描
pub async fn mfa_bind_qrcode(
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    // 获取应用名称
    let app_name = if auth_data.user().app_id == 0 {
        "lsys".to_string()
    } else {
        // 获取应用信息
        let app = web_dao
            .web_app
            .app_dao
            .app
            .cache()
            .find_by_id(auth_data.user().app_id)
            .await?;
        app.name.clone()
    };

    // 生成新的Secret - 使用Base32标准字母表 (A-Z, 2-7)
    // TOTP标准要求Secret是Base32编码的字符串
    let base32_chars = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut rng = rand::rng();
    let secret: String = base32_chars
        .sample(&mut rng, 32)
        .map(|&b| b as char)
        .collect();

    // 生成TOTP标准的otpauth://协议URL
    // 格式: otpauth://totp/[issuer:]user?secret=SECRET&issuer=ISSUER
    // 注意: issuer/label 需要做URL百分号编码，避免空格、中文、冒号等特殊字符破坏URL
    let url_encode = |input: &str| -> String {
        url::form_urlencoded::byte_serialize(input.as_bytes()).collect()
    };

    let issuer_enc = url_encode(&app_name);
    let user_enc = url_encode(&auth_data.user().user_nickname);
    let secret_enc = url_encode(&secret);
    let otpauth_url = format!(
        "otpauth://totp/{}:{}?secret={}&issuer={}",
        issuer_enc, user_enc, secret_enc, issuer_enc
    );
    let len = web_dao
        .app_core
        .config
        .find(None)
        .get_int("mfa_totp_digits")
        .unwrap_or(6);
    // 生成二维码数据 - 这里返回OTP Auth URL，前端可以使用第三方库生成二维码
    // 或者使用某个二维码生成服务
    Ok(JsonResponse::data(JsonData::body(json!({
        "secret": secret,
        "len":len,
        "otpauth_url": otpauth_url,
        "app_name": app_name,
    }))))
}

/// MFA绑定验证参数
#[derive(Debug, Deserialize)]
pub struct MfaBindParam {
    /// Base32编码的Secret密钥
    pub secret: String,
    /// MFA设备显示的TOTP验证码
    pub code: String,
}

/// 绑定MFA设备接口
/// 用户在MFA设备扫描二维码后，输入显示的TOTP验证码进行绑定
pub async fn mfa_bind_device(
    param: &MfaBindParam,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    // 创建MFA主体
    let subject = MfaSubject {
        app_id: auth_data.user().app_id,
        user_data: auth_data.user().user_data.clone(),
    };

    // 在启用新Secret前先验证TOTP码是否正确
    // 这确保用户确实拥有该MFA设备
    let totp_dao = &web_dao.web_mfa.totp_dao;

    // 验证Secret格式并解码
    let secret_key = lsys_mfa::dao::decode_base32(&param.secret)?;

    // 获取TOTP配置
    let config = lsys_mfa::dao::MfaTotpConfig::load(&web_dao.app_core);

    // 验证TOTP码 - 检查用户输入的验证码是否正确
    let code_trimmed = param.code.trim();
    let now = lsys_core::utils::now_time().unwrap_or_default();
    let now_step = now / config.step_seconds;

    // 检查time window内的所有可能的TOTP值
    let mut code_valid = false;
    for offset in -config.window..=config.window {
        let step = if offset.is_negative() {
            now_step.saturating_sub(offset.unsigned_abs())
        } else {
            now_step.saturating_add(offset as u64)
        };
        let generated_code = lsys_mfa::dao::totp_code(&secret_key, step, config.digits)?;

        if generated_code == code_trimmed {
            code_valid = true;
            break;
        }
    }

    if !code_valid {
        return Err(crate::common::JsonError::Message(fluent_message!(
            "mfa-code-invalid"
        )));
    }

    // 验证成功，启用新的TOTP Secret
    totp_dao.enable_new_secret(&subject, &param.secret).await?;

    Ok(JsonResponse::default())
}

/// 获取用户MFA绑定状态
pub async fn mfa_status(auth_dao: &UserAuthQueryDao, web_dao: &WebDao) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    let subject = MfaSubject {
        app_id: auth_data.user().app_id,
        user_data: auth_data.user().user_data.clone(),
    };

    let is_enabled = web_dao.web_mfa.totp_dao.is_enabled(&subject).await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "enabled": is_enabled,
    }))))
}

/// 解绑MFA设备
pub async fn mfa_unbind(auth_dao: &UserAuthQueryDao, web_dao: &WebDao) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;
    let subject = MfaSubject {
        app_id: auth_data.user().app_id,
        user_data: auth_data.user().user_data.clone(),
    };
    // 禁用MFA
    web_dao.web_mfa.totp_dao.disable(&subject).await?;
    Ok(JsonResponse::default())
}
