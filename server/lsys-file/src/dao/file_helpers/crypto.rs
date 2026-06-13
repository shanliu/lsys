use lsys_core::fluents::IntoFluentMessage;

use crate::common as crypto;

/// 用于加密/解密操作的 config key，对应 `[secret.file_aes_key]` 配置段。
pub const CRYPTO_SECRET_KEY_ID: &str = "file_aes_key";

impl super::FileHelper {
    /// 加密文件
    ///
    /// # Arguments
    /// * `source_path` - 待加密文件路径
    ///
    /// # Returns
    /// * `Ok((relative_path, full_path))` - 加密成功，返回相对路径和完整路径
    /// * `Err(std::io::Error)` - 加密失败
    pub async fn encrypt_new_file(
        &self,
        source_path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<(String, std::path::PathBuf)> {
        let key = self
            .secret_manager
            .require(CRYPTO_SECRET_KEY_ID)
            .map_err(|e| std::io::Error::other(e.to_fluent_message().default_format()))?;
        let source = source_path.as_ref();
        let ext = source.extension().and_then(|e| e.to_str()).unwrap_or("enc");
        let (relative_path, full_path) = self
            .create_new_file(
                crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO,
                "enc",
                ext,
            )
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        crypto::encrypt_file(key, source, &full_path).await?;
        Ok((relative_path, full_path))
    }

    /// 解密文件片段（创建异步迭代器）
    ///
    /// # Arguments
    /// * `relative_path` - 加密文件的相对路径
    /// * `offset` - 读取起始位置（相对于原始文件）
    /// * `length` - 读取长度，None 表示读取到文件末尾
    ///
    /// # Returns
    /// * `Ok(DecryptIterator)` - 解密异步迭代器
    /// * `Err(std::io::Error)` - 创建失败
    pub async fn decrypt_file_range(
        &self,
        relative_path: impl AsRef<std::path::Path>,
        offset: u64,
        length: Option<u64>,
    ) -> std::io::Result<crate::common::DecryptIterator> {
        let key = self
            .secret_manager
            .require(CRYPTO_SECRET_KEY_ID)
            .map_err(|e| std::io::Error::other(e.to_fluent_message().default_format()))?;
        let crypto_base = self
            .config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let full_path = crypto_base.join(relative_path.as_ref());
        crypto::decrypt_file_range(key, full_path, offset, length).await
    }

    /// 获取加密文件的原始大小
    ///
    /// # Arguments
    /// * `relative_path` - 加密文件的相对路径
    ///
    /// # Returns
    /// * `Ok(u64)` - 原始文件大小
    /// * `Err(std::io::Error)` - 获取失败
    pub async fn get_encrypted_file_size(
        &self,
        relative_path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<u64> {
        let crypto_base = self
            .config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let full_path = crypto_base.join(relative_path.as_ref());
        crypto::get_decrypted_size(full_path).await
    }

    /// 验证加密文件是否有效
    ///
    /// # Arguments
    /// * `relative_path` - 加密文件的相对路径
    ///
    /// # Returns
    /// * `Ok(true)` - 文件有效
    /// * `Ok(false)` - 文件无效
    /// * `Err(std::io::Error)` - 验证失败
    pub async fn verify_encrypted_file(
        &self,
        relative_path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<bool> {
        let key = self
            .secret_manager
            .require(CRYPTO_SECRET_KEY_ID)
            .map_err(|e| std::io::Error::other(e.to_fluent_message().default_format()))?;
        let crypto_base = self
            .config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let full_path = crypto_base.join(relative_path.as_ref());
        crypto::verify_encrypted_file(key, full_path).await
    }

    /// 解密文件到指定存储类型
    ///
    /// 文件路径由 `create_new_file` 统一按日期目录生成，外部无需指定文件名。
    ///
    /// # Arguments
    /// * `encrypted_relative_path` - 加密文件的相对路径
    /// * `storage_type` - 目标存储类型（STORAGE_TYPE_LOCAL_PUBLIC 或 STORAGE_TYPE_LOCAL_PRIVATE）
    /// * `original_ext` - 原始文件扩展名（可选，默认为 "dat"）
    ///
    /// # Returns
    /// * `Ok((relative_path, full_path))` - 解密成功，返回相对路径和完整路径
    /// * `Err(std::io::Error)` - 解密失败
    pub async fn decrypt_file_to_storage(
        &self,
        encrypted_relative_path: impl AsRef<std::path::Path>,
        storage_type: &str,
        original_ext: Option<&str>,
    ) -> std::io::Result<(String, std::path::PathBuf)> {
        let key = self
            .secret_manager
            .require(CRYPTO_SECRET_KEY_ID)
            .map_err(|e| std::io::Error::other(e.to_fluent_message().default_format()))?;
        let ext = original_ext.unwrap_or("dat");
        let (relative_path, full_path) = self
            .create_new_file(storage_type, "dec", ext)
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;

        let crypto_base = self
            .config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let encrypted_full_path = crypto_base.join(encrypted_relative_path.as_ref());
        crypto::decrypt_file_to(key, encrypted_full_path, &full_path).await?;
        Ok((relative_path, full_path))
    }

    /// 解密文件到系统临时目录（不入库）
    ///
    /// 返回临时文件路径，调用方在使用完毕后应自行删除该文件。
    ///
    /// # Arguments
    /// * `encrypted_relative_path` - 加密文件的相对路径
    /// * `ext` - 临时文件扩展名（如 "dat"）
    pub async fn decrypt_to_temp_file(
        &self,
        encrypted_relative_path: impl AsRef<std::path::Path>,
        ext: &str,
    ) -> std::io::Result<std::path::PathBuf> {
        let key = self
            .secret_manager
            .require(CRYPTO_SECRET_KEY_ID)
            .map_err(|e| std::io::Error::other(e.to_fluent_message().default_format()))?;
        let crypto_base = self
            .config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let encrypted_full_path = crypto_base.join(encrypted_relative_path.as_ref());

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let file_name = format!("lsys_dec_{}_{}.{}", std::process::id(), ts, ext);
        let temp_path = std::env::temp_dir().join(file_name);
        crypto::decrypt_file_to(key, &encrypted_full_path, &temp_path).await?;
        Ok(temp_path)
    }
}
