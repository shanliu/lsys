//! lsys-kms 集成测试

#[cfg(test)]
mod tests {
    use lsys_core::{fluents::IntoFluentMessage, secret::SecretError};

    #[test]
    fn test_secret_error_display() {
        let error = SecretError::KeyNotFound("test_key".to_string());
        let message = error.to_fluent_message().default_format();
        assert!(!message.is_empty());
    }

    #[test]
    fn test_kms_decode_scenario() {
        use base64::Engine;
        let plaintext = "my-secret-password";
        let encoded = base64::engine::general_purpose::STANDARD.encode(plaintext);
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .expect("decode failed");
        assert_eq!(String::from_utf8(decoded).unwrap(), plaintext);
    }
}

#[cfg(all(test, feature = "aliyun-kms"))]
mod aliyun_tests {
    use lsys_kms::aliyun::AliyunKmsDecryptor;

    #[test]
    fn test_aliyun_endpoint_construction() {
        let cases = [
            ("cn-beijing", "https://kms.cn-beijing.aliyuncs.com"),
            ("cn-shanghai", "https://kms.cn-shanghai.aliyuncs.com"),
            ("ap-southeast-1", "https://kms.ap-southeast-1.aliyuncs.com"),
        ];
        for (region, expected) in cases {
            let d = AliyunKmsDecryptor::new("test-id", "test-secret", region);
            assert_eq!(d.get_endpoint(), expected);
        }
    }
}

#[cfg(all(test, feature = "tencent-kms"))]
mod tencent_tests {
    use lsys_kms::tencent::TencentKmsDecryptor;

    #[test]
    fn test_tencent_endpoint_construction() {
        let d = TencentKmsDecryptor::new("test-id", "test-key", "ap-beijing");
        assert_eq!(d.get_endpoint(), "https://kms.tencentcloudapi.com");
    }

    #[test]
    fn test_tencent_region_support() {
        for region in ["ap-beijing", "ap-shanghai", "ap-hongkong"] {
            let _d = TencentKmsDecryptor::new("id", "key", region);
        }
    }
}

