use crate::{dao::FileResult, model::FileModel};

use super::FileDataDao;

impl FileDataDao {
    /// 判断指定的 FileModel 是否是私有的
    ///
    /// 私有文件包括：
    /// - 本地存储：storage_type 为 "local_private" 或 "local_crypto"
    /// - OSS 存储：查询 OSS 配置中的 is_private 字段
    ///
    /// 如果是 OSS 文件但配置不存在，默认视为私有（安全起见）
    ///
    /// # 参数
    /// - `file`: 要判断的文件模型
    ///
    /// # 返回
    /// - `Ok(true)`: 文件是私有的
    /// - `Ok(false)`: 文件是公开的
    /// - `Err(...)`: 查询 OSS 配置时发生错误
    ///
    /// # 示例
    /// ```rust,ignore
    /// let file_data_dao = FileDataDao::new(...);
    /// let file: FileModel = ...;
    ///
    /// match file_data_dao.is_private(&file).await {
    ///     Ok(true) => println!("私有文件"),
    ///     Ok(false) => println!("公开文件"),
    ///     Err(e) => eprintln!("查询失败: {}", e),
    /// }
    /// ```
    pub async fn is_private(&self, file: &FileModel) -> FileResult<bool> {
        // 本地私有存储类型判断
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_PRIVATE
            || file.storage_type == FileModel::STORAGE_TYPE_LOCAL_CRYPTO
        {
            return Ok(true);
        }

        // 本地公开存储
        if file.storage_type == FileModel::STORAGE_TYPE_LOCAL_PUBLIC {
            return Ok(false);
        }

        // OSS 存储：查询配置中的 is_private 字段
        match self
            .oss_config
            .find_by_config_key(&file.storage_type)
            .await?
        {
            Some(config) => Ok(config.is_private),
            // 配置不存在，默认视为私有（安全起见）
            None => Ok(true),
        }
    }
}
