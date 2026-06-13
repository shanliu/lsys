use lsys_kms::aliyun::AliyunKmsDecryptor;

/// 阿里云 KMS 使用示例
///
/// 运行此示例前，请确保：
/// 1. 已启用 `aliyun-kms` 特性
/// 2. 已配置 lsys-core 的 config
/// 3. 已在配置文件中定义密钥
///
/// cargo run --example aliyun_kms --features aliyun-kms


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 阿里云 KMS 解密器示例 ===\n");

    // 创建阿里云 KMS 解密器实例
    // 注意：实际使用时应该从环境变量或配置中读取凭证
    let _aliyun_kms = AliyunKmsDecryptor::new(
        std::env::var("ALIYUN_ACCESS_KEY_ID")
            .unwrap_or_else(|_| "your-access-key-id".to_string()),
        std::env::var("ALIYUN_ACCESS_KEY_SECRET")
            .unwrap_or_else(|_| "your-access-key-secret".to_string()),
        "cn-beijing",
    );

    println!("✓ 创建了阿里云 KMS 解密器实例");
    println!("  Region: cn-beijing\n");

    // 在实际应用中，你会这样使用：
    // let config = lsys_core::config::Config::load("config.toml")?;
    // let manager = SecretManager::builder(&config)
    //     .kms_provider("aliyun", Arc::new(aliyun_kms))
    //     .build()
    //     .await?;
    //
    // let password = manager.require_str("database_password")?;
    // println!("数据库密码: {}", password);

    println!("配置示例:");
    println!("```toml");
    println!("[secret.database_password]");
    println!("source     = \"kms\"");
    println!("kms        = \"aliyun\"");
    println!("ciphertext = \"base64:i4+7vLQnj2kLe5...\"");
    println!("```");

    Ok(())
}
