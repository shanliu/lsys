/// 腾讯云 KMS 使用示例
///
/// 运行此示例前，请确保：
/// 1. 已启用 `tencent-kms` 特性
/// 2. 已配置 lsys-core 的 config
/// 3. 已在配置文件中定义密钥
///
/// cargo run --example tencent_kms --features tencent-kms
use lsys_kms::tencent::TencentKmsDecryptor;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 腾讯云 KMS 解密器示例 ===\n");

    // 创建腾讯云 KMS 解密器实例
    // 注意：实际使用时应该从环境变量或配置中读取凭证
    let _tencent_kms = TencentKmsDecryptor::new(
        std::env::var("TENCENT_SECRET_ID")
            .unwrap_or_else(|_| "your-secret-id".to_string()),
        std::env::var("TENCENT_SECRET_KEY")
            .unwrap_or_else(|_| "your-secret-key".to_string()),
        "ap-beijing",
    );

    println!("✓ 创建了腾讯云 KMS 解密器实例");
    println!("  Region: ap-beijing\n");

    // 在实际应用中，你会这样使用：
    // let config = lsys_core::config::Config::load("config.toml")?;
    // let manager = SecretManager::builder(&config)
    //     .kms_provider("tencent", Arc::new(tencent_kms))
    //     .build()
    //     .await?;
    //
    // let api_key = manager.require_str("api_key")?;
    // println!("API Key: {}", api_key);

    println!("配置示例:");
    println!("```toml");
    println!("[secret.api_key]");
    println!("source     = \"kms\"");
    println!("kms        = \"tencent\"");
    println!("ciphertext = \"base64:j5/8wMRoj3lMf6...\"");
    println!("```");

    Ok(())
}
