/// 多 KMS 提供商使用示例
///
/// 演示如何同时使用阿里云和腾讯云的 KMS 服务来解密不同的密钥
///
/// cargo run --example multi_kms --features aliyun-kms,tencent-kms
use lsys_kms::aliyun::AliyunKmsDecryptor;
use lsys_kms::tencent::TencentKmsDecryptor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 多 KMS 提供商示例 ===\n");

    // 创建阿里云 KMS 解密器
    let _aliyun_kms = AliyunKmsDecryptor::new(
        std::env::var("ALIYUN_ACCESS_KEY_ID")
            .unwrap_or_else(|_| "your-aliyun-access-key-id".to_string()),
        std::env::var("ALIYUN_ACCESS_KEY_SECRET")
            .unwrap_or_else(|_| "your-aliyun-access-key-secret".to_string()),
        "cn-beijing",
    );

    // 创建腾讯云 KMS 解密器
    let _tencent_kms = TencentKmsDecryptor::new(
        std::env::var("TENCENT_SECRET_ID")
            .unwrap_or_else(|_| "your-tencent-secret-id".to_string()),
        std::env::var("TENCENT_SECRET_KEY")
            .unwrap_or_else(|_| "your-tencent-secret-key".to_string()),
        "ap-beijing",
    );

    println!("✓ 创建了阿里云 KMS 解密器 (cn-beijing)");
    println!("✓ 创建了腾讯云 KMS 解密器 (ap-beijing)\n");

    // 在实际应用中，你会这样使用：
    // let config = lsys_core::config::Config::load("config.toml")?;
    // let manager = SecretManager::builder(&config)
    //     .kms_provider("aliyun", Arc::new(aliyun_kms))
    //     .kms_provider("tencent", Arc::new(tencent_kms))
    //     .build()
    //     .await?;
    //
    // // 使用不同的 KMS 解密不同的密钥
    // let db_password = manager.require_str("database_password")?;  // 使用 aliyun
    // let api_key = manager.require_str("api_key")?;                 // 使用 tencent
    // let cache_password = manager.require_str("cache_password")?;   // 使用 aliyun

    println!("配置示例:");
    println!("```toml");
    println!("# 数据库密钥 - 使用阿里云 KMS");
    println!("[secret.database_password]");
    println!("source     = \"kms\"");
    println!("kms        = \"aliyun\"");
    println!("ciphertext = \"base64:...\"");
    println!();
    println!("# API 密钥 - 使用腾讯云 KMS");
    println!("[secret.api_key]");
    println!("source     = \"kms\"");
    println!("kms        = \"tencent\"");
    println!("ciphertext = \"base64:...\"");
    println!();
    println!("# 缓存密钥 - 使用阿里云 KMS");
    println!("[secret.cache_password]");
    println!("source     = \"kms\"");
    println!("kms        = \"aliyun\"");
    println!("ciphertext = \"base64:...\"");
    println!("```");

    println!("\n这个配置允许你为不同的密钥使用不同的 KMS 提供商！");

    Ok(())
}
