use sqlx::{MySql, Pool};

use super::file_config::FileConfig;
use crate::common as crypto;

/// 辅助函数集合
pub struct FileHelper {
    pub(super) db: Pool<MySql>,
    pub(super) config: FileConfig,
}

impl FileHelper {
    pub fn new(db: Pool<MySql>, config: FileConfig) -> Self {
        Self { db, config }
    }

    /// 加密文件
    /// 
    /// # Arguments
    /// * `source_path` - 待加密文件路径
    /// 
    /// # Returns
    /// * `Ok((relative_path, full_path))` - 加密成功，返回相对路径和完整路径
    /// * `Err(std::io::Error)` - 加密失败
    pub async fn encrypt_file(
        &self,
        source_path: impl AsRef<std::path::Path>,
    ) -> std::io::Result<(String, std::path::PathBuf)> {
        let source = source_path.as_ref();
        let ext = source
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("enc");
        let (relative_path, full_path) = self
            .create_new_file(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO, "enc", ext)
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        crypto::encrypt_file(source, &full_path).await?;
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
        let crypto_base = self.config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let full_path = crypto_base.join(relative_path.as_ref());
        crypto::decrypt_file_range(full_path, offset, length).await
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
        let crypto_base = self.config
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
        let crypto_base = self.config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let full_path = crypto_base.join(relative_path.as_ref());
        crypto::verify_encrypted_file(full_path).await
    }

    /// 解密文件到指定存储类型
    /// 
    /// 文件路径由 `create_new_file` 统一按日期目录生成，外部无需指定文件名。
    /// 
    /// # Arguments
    /// * `encrypted_relative_path` - 加密文件的相对路径
    /// * `storage_type` - 目标存储类型（STORAGE_TYPE_LOCAL_PUBLIC 或 STORAGE_TYPE_LOCAL_PRIVATE）
    /// 
    /// # Returns
    /// * `Ok((relative_path, full_path))` - 解密成功，返回相对路径和完整路径
    /// * `Err(std::io::Error)` - 解密失败
    pub async fn decrypt_file_to_storage(
        &self,
        encrypted_relative_path: impl AsRef<std::path::Path>,
        storage_type: &str,
    ) -> std::io::Result<(String, std::path::PathBuf)> {
        let (relative_path, full_path) = self
            .create_new_file(storage_type, "dec", "dat")
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
        
        let crypto_base = self.config
            .get_base_path(crate::model::FileModel::STORAGE_TYPE_LOCAL_CRYPTO)
            .await?;
        let encrypted_full_path = crypto_base.join(encrypted_relative_path.as_ref());
        crypto::decrypt_file_to(encrypted_full_path, &full_path).await?;
        Ok((relative_path, full_path))
    }
}

mod chunk;
mod complete;
mod http;
mod ops;
mod query;

// Re-export types
pub use chunk::ChunkInfo;
pub use http::UrlFileInfo;
