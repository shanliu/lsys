use std::path::PathBuf;

use bytes::Bytes;
use futures_util::StreamExt;
use lsys_core::fluent_message;
use lsys_core::fluents::IntoFluentMessage;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt};

use super::FileDataDao;
use super::*;
use crate::common::crypto::{DecryptIterator, decrypt_file_range};
use crate::model::*;

// ==================== 公共类型定义 ====================

/// 文件读取块信息
#[derive(Debug, Clone)]
pub struct FileReadChunk {
    /// 块索引
    pub index: usize,
    /// 块数据
    pub data: Vec<u8>,
    /// 块在文件中的偏移
    pub offset: u64,
    /// 块大小
    pub size: usize,
}

/// 文件读取迭代器（统一接口）
///
/// 支持本地文件（public/private/crypto）的流式读取，自动处理加密解密。
pub enum FileReadIterator {
    /// 普通文件读取器（local_public / local_private）
    Plain(PlainFileIterator),
    /// 加密文件读取器（local_crypto）
    Encrypted(DecryptIterator),
}

impl FileReadIterator {
    /// 异步获取下一个数据块
    pub async fn next_chunk(&mut self) -> Option<std::io::Result<FileReadChunk>> {
        match self {
            FileReadIterator::Plain(iter) => iter.next_chunk().await,
            FileReadIterator::Encrypted(iter) => iter.next_chunk().await.map(|result| {
                result.map(|(chunk_block, data)| FileReadChunk {
                    index: chunk_block.index,
                    data,
                    offset: chunk_block.file_offset + chunk_block.read_start as u64,
                    size: chunk_block.read_len,
                })
            }),
        }
    }

    /// 获取读取范围信息 (start_offset, end_offset, total_length)
    pub fn range_info(&self) -> (u64, u64, u64) {
        match self {
            FileReadIterator::Plain(iter) => iter.range_info(),
            FileReadIterator::Encrypted(iter) => iter.range_info(),
        }
    }
}

/// 普通文件读取迭代器（非加密）
pub struct PlainFileIterator {
    file: File,
    start_offset: u64,
    end_offset: u64,
    current_offset: u64,
    chunk_size: usize,
    chunk_index: usize,
}

impl PlainFileIterator {
    async fn new(
        file_path: PathBuf,
        start_offset: u64,
        end_offset: u64,
        chunk_size: usize,
    ) -> std::io::Result<Self> {
        let mut file = File::open(file_path).await?;
        file.seek(tokio::io::SeekFrom::Start(start_offset)).await?;

        Ok(Self {
            file,
            start_offset,
            end_offset,
            current_offset: start_offset,
            chunk_size,
            chunk_index: 0,
        })
    }

    pub fn range_info(&self) -> (u64, u64, u64) {
        (
            self.start_offset,
            self.end_offset,
            self.end_offset - self.start_offset,
        )
    }

    async fn next_chunk(&mut self) -> Option<std::io::Result<FileReadChunk>> {
        if self.current_offset >= self.end_offset {
            return None;
        }

        let remaining = self.end_offset - self.current_offset;
        let read_size = std::cmp::min(remaining, self.chunk_size as u64) as usize;

        let mut buffer = vec![0u8; read_size];
        match self.file.read_exact(&mut buffer).await {
            Ok(_) => {
                let chunk = FileReadChunk {
                    index: self.chunk_index,
                    data: buffer,
                    offset: self.current_offset,
                    size: read_size,
                };
                self.current_offset += read_size as u64;
                self.chunk_index += 1;
                Some(Ok(chunk))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

/// 统一文件读取流
///
/// 支持本地文件和 OSS 文件的统一流式读取接口。
pub enum UnifiedFileStream {
    /// 本地文件流
    Local(FileReadIterator),
    /// OSS 文件流（支持 Range）
    OssRangeSupported(crate::common::OssDownloadStream),
    /// OSS 文件流（不支持 Range，需要客户端处理偏移）
    OssFullStream {
        stream: crate::common::OssDownloadStream,
        /// 需要跳过的字节数
        skip_bytes: u64,
        /// 已跳过的字节数
        skipped: u64,
        /// 需要读取的总字节数（None 表示读到末尾）
        read_limit: Option<u64>,
        /// 已读取的字节数
        read_bytes: u64,
    },
}

impl UnifiedFileStream {
    /// 获取下一个数据块（统一为 Bytes 格式）
    pub async fn next_bytes(&mut self) -> Option<FileResult<Bytes>> {
        match self {
            UnifiedFileStream::Local(iter) => iter.next_chunk().await.map(|result| {
                result
                    .map(|chunk| Bytes::from(chunk.data))
                    .map_err(FileError::from)
            }),
            UnifiedFileStream::OssRangeSupported(stream) => stream
                .next()
                .await
                .map(|result| result.map(|chunk| chunk.data)),
            UnifiedFileStream::OssFullStream {
                stream,
                skip_bytes,
                skipped,
                read_limit,
                read_bytes,
            } => {
                // 处理需要跳过和限制读取的情况
                loop {
                    // 检查是否已达到读取限制
                    if let Some(limit) = read_limit
                        && *read_bytes >= *limit {
                            return None;
                        }

                    let chunk_result = stream.next().await?;
                    let chunk = match chunk_result {
                        Ok(c) => c,
                        Err(e) => return Some(Err(e)),
                    };

                    let chunk_len = chunk.data.len() as u64;

                    // 如果还需要跳过数据
                    if *skipped < *skip_bytes {
                        let remaining_skip = *skip_bytes - *skipped;
                        if chunk_len <= remaining_skip {
                            // 整个块都需要跳过
                            *skipped += chunk_len;
                            continue;
                        } else {
                            // 跳过部分数据，返回剩余部分
                            *skipped = *skip_bytes;
                            let start_pos = remaining_skip as usize;
                            let mut data = chunk.data.slice(start_pos..);

                            // 检查读取限制
                            if let Some(limit) = read_limit {
                                let remaining_read = *limit - *read_bytes;
                                if data.len() as u64 > remaining_read {
                                    data = data.slice(..remaining_read as usize);
                                }
                            }

                            *read_bytes += data.len() as u64;
                            return Some(Ok(data));
                        }
                    } else {
                        // 不需要跳过，直接返回数据
                        let mut data = chunk.data;

                        // 检查读取限制
                        if let Some(limit) = read_limit {
                            let remaining_read = *limit - *read_bytes;
                            if data.len() as u64 > remaining_read {
                                data = data.slice(..remaining_read as usize);
                            }
                        }

                        *read_bytes += data.len() as u64;
                        return Some(Ok(data));
                    }
                }
            }
        }
    }
}

// ==================== FileDataDao 读取功能实现 ====================

impl FileDataDao {
    /// 读取本地文件（支持偏移）
    ///
    /// 返回本地文件的流式读取迭代器，支持加密文件自动解密。
    ///
    /// # 参数
    /// - `file`: 文件模型（必须是本地文件）
    /// - `offset`: 读取起始偏移（字节）
    /// - `length`: 读取长度，None 表示读取到文件末尾
    ///
    /// # 返回
    /// - `Ok(FileReadIterator)`: 文件读取迭代器
    /// - `Err`: 文件不是本地文件、文件不存在、或其他 IO 错误
    ///
    /// # 示例
    /// ```rust,ignore
    /// let mut iter = file_data_dao.read_local_file(&file_model, 0, None).await?;
    /// while let Some(result) = iter.next_chunk().await {
    ///     let chunk = result?;
    ///     // 处理 chunk.data
    /// }
    /// ```
    pub async fn read_local_file(
        &self,
        file: &FileModel,
        offset: u64,
        length: Option<u64>,
    ) -> FileResult<FileReadIterator> {
        // 检查文件状态
        if !FileStatus::Normal.eq(file.status) {
            return Err(FileError::System(fluent_message!(
                "file-error",
                &format!("File status is not normal: {}", file.status)
            )));
        }

        // 查询 file_local 记录获取文件路径
        let local_record = self.get_file_local_record_for_read(file.id).await?;
        self.read_local_file_from_record(file, &local_record, offset, length)
            .await
    }

    /// 从 file_local 记录读取本地文件（支持偏移）
    /// 被 read_local_file 或 Cache 层调用，避免重复查库
    pub async fn read_local_file_from_record(
        &self,
        file: &FileModel,
        local_record: &FileLocalModel,
        offset: u64,
        length: Option<u64>,
    ) -> FileResult<FileReadIterator> {
        // 只支持本地文件
        if !file.is_local() {
            return Err(FileError::System(fluent_message!(
                "file-error",
                "File is not local type, use read_oss_file instead"
            )));
        }

        // 构建完整文件路径
        let file_path = self.build_file_path_for_read(file, local_record).await?;

        // 验证文件存在
        if !tokio::fs::try_exists(&file_path).await? {
            return Err(FileError::System(fluent_message!(
                "file-error",
                &format!("File not found: {}", file_path.display())
            )));
        }

        // 获取文件大小
        let metadata = tokio::fs::metadata(&file_path).await?;
        let file_size = metadata.len();

        // 计算实际读取范围
        let actual_length = length.unwrap_or(file_size.saturating_sub(offset));
        let end_offset = offset + actual_length;

        if end_offset > file_size {
            return Err(FileError::System(fluent_message!(
                "file-error",
                &format!(
                    "Read range exceeds file size: offset={}, length={}, file_size={}",
                    offset, actual_length, file_size
                )
            )));
        }

        if actual_length == 0 {
            return Err(FileError::System(fluent_message!(
                "file-error",
                "Read length cannot be zero"
            )));
        }

        // 根据存储类型创建对应的迭代器
        match file.storage_type.as_str() {
            FileModel::STORAGE_TYPE_LOCAL_CRYPTO => {
                // 加密文件：使用 DecryptIterator
                let key = self
                    .helper
                    .secret_manager
                    .require(crate::dao::file_helpers::CRYPTO_SECRET_KEY_ID)
                    .map_err(|e| {
                        crate::dao::FileError::System(e.to_fluent_message())
                    })?;
                let iter = decrypt_file_range(key, &file_path, offset, Some(actual_length)).await?;
                Ok(FileReadIterator::Encrypted(iter))
            }
            FileModel::STORAGE_TYPE_LOCAL_PUBLIC | FileModel::STORAGE_TYPE_LOCAL_PRIVATE => {
                // 普通文件：使用 PlainFileIterator
                // 使用 1MB 作为默认块大小
                const DEFAULT_CHUNK_SIZE: usize = 1024 * 1024;
                let iter =
                    PlainFileIterator::new(file_path, offset, end_offset, DEFAULT_CHUNK_SIZE)
                        .await?;
                Ok(FileReadIterator::Plain(iter))
            }
            _ => Err(FileError::System(fluent_message!(
                "file-error",
                &format!("Unsupported storage type: {}", file.storage_type)
            ))),
        }
    }

    /// 读取 OSS 文件（支持偏移）
    ///
    /// 返回 OSS 文件的流式下载结果，支持边下载边读取。
    ///
    /// # 参数
    /// - `file`: 文件模型（必须是 OSS 文件）
    /// - `offset`: 读取起始偏移（字节）
    /// - `length`: 读取长度，None 表示读取到文件末尾
    ///
    /// # 返回
    /// - `Ok(OssDownloadResult)`: OSS 下载结果
    ///   - `RangeSupported`: 支持偏移读取，流已从指定位置开始
    ///   - `FullStreamOnly`: 不支持偏移读取，返回完整文件流，调用者需要在客户端跳过前面的数据
    /// - `Err`: 文件不是 OSS 文件、配置错误、或其他错误
    ///
    /// # 示例 1: 处理支持 Range 的 OSS
    /// ```rust,ignore
    /// use futures_util::StreamExt;
    ///
    /// let result = file_data_dao.read_oss_file(&file_model, 1024, Some(2048)).await?;
    /// match result {
    ///     OssDownloadResult::RangeSupported(mut stream) => {
    ///         // 直接使用，流已从偏移位置开始
    ///         while let Some(result) = stream.next().await {
    ///             let chunk = result?;
    ///             // 处理 chunk.data
    ///         }
    ///     }
    ///     OssDownloadResult::FullStreamOnly(mut stream) => {
    ///         // 需要在客户端跳过前 1024 字节
    ///         let mut skipped = 0u64;
    ///         let mut read = 0u64;
    ///         while let Some(result) = stream.next().await {
    ///             let chunk = result?;
    ///             if skipped < 1024 {
    ///                 let skip_in_chunk = std::cmp::min(chunk.data.len() as u64, 1024 - skipped);
    ///                 skipped += skip_in_chunk;
    ///                 if skip_in_chunk < chunk.data.len() as u64 {
    ///                     // 处理剩余数据
    ///                     let remaining = &chunk.data[skip_in_chunk as usize..];
    ///                     read += remaining.len() as u64;
    ///                     // 处理 remaining
    ///                 }
    ///             } else {
    ///                 read += chunk.data.len() as u64;
    ///                 // 处理 chunk.data
    ///             }
    ///             if read >= 2048 {
    ///                 break;
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    pub async fn read_oss_file(
        &self,
        file: &FileModel,
        offset: u64,
        length: Option<u64>,
    ) -> FileResult<crate::common::OssDownloadResult> {
        // 检查文件状态
        if !FileStatus::Normal.eq(file.status) {
            return Err(FileError::System(fluent_message!(
                "file-error",
                &format!("File status is not normal: {}", file.status)
            )));
        }

        // 查询 file_oss 记录
        let file_oss = self.get_file_oss_record(file.id).await?;
        self.read_oss_file_from_record(file, &file_oss, offset, length)
            .await
    }

    /// 从 file_oss 记录读取 OSS 文件（支持偏移）
    /// 被 read_oss_file 或 Cache 层调用，避免重复查库
    pub async fn read_oss_file_from_record(
        &self,
        file: &FileModel,
        file_oss: &FileOssModel,
        offset: u64,
        length: Option<u64>,
    ) -> FileResult<crate::common::OssDownloadResult> {
        // 只支持 OSS 文件
        if !file.is_oss() {
            return Err(FileError::System(fluent_message!(
                "file-error",
                "File is not OSS type, use read_local_file instead"
            )));
        }

        // 获取 OSS 配置
        let config_key = file.oss_config_key().ok_or_else(|| {
            FileError::System(fluent_message!(
                "file-error",
                &format!("File is not OSS type: {}", file.storage_type)
            ))
        })?;

        let provider = self.oss_config.resolve_provider(config_key).await?;

        // 创建流式下载，返回结果（让调用者知道是否支持 Range）
        let result = provider
            .download_stream(file_oss, Some(offset), length)
            .await?;

        Ok(result)
    }

    /// 读取文件（自动判断本地/OSS，支持偏移）
    ///
    /// 统一的文件读取接口，自动判断文件类型（本地/OSS）并返回对应的流。
    /// 适用于需要直接输出到用户端的场景，支持边下载边返回。
    /// 自动处理不支持 Range 请求的 OSS，在客户端进行偏移和长度限制。
    ///
    /// # 参数
    /// - `file`: 文件模型
    /// - `offset`: 读取起始偏移（字节）
    /// - `length`: 读取长度，None 表示读取到文件末尾
    ///
    /// # 返回
    /// - `Ok(UnifiedFileStream)`: 统一文件流（可能是本地或 OSS）
    /// - `Err`: 文件不存在、配置错误、或其他错误
    ///
    /// # 示例
    /// ```rust,ignore
    /// let mut stream = file_data_dao.read_file(&file_model, 1024, Some(2048)).await?;
    /// while let Some(result) = stream.next_bytes().await {
    ///     let bytes = result?;
    ///     // 写入响应流
    ///     response.write_all(&bytes).await?;
    /// }
    /// ```
    pub async fn read_file(
        &self,
        file: &FileModel,
        offset: u64,
        length: Option<u64>,
    ) -> FileResult<UnifiedFileStream> {
        // 检查文件状态
        if !FileStatus::Normal.eq(file.status) {
            return Err(FileError::System(fluent_message!(
                "file-error",
                &format!("File status is not normal: {}", file.status)
            )));
        }

        if file.is_local() {
            // 本地文件：使用 read_local_file 方法
            let iter = self.read_local_file(file, offset, length).await?;
            Ok(UnifiedFileStream::Local(iter))
        } else {
            // OSS 文件：使用 read_oss_file 方法
            let result = self.read_oss_file(file, offset, length).await?;

            // 根据 OSS 是否支持 Range 创建不同的流
            match result {
                crate::common::OssDownloadResult::RangeSupported(stream) => {
                    // 支持 Range，直接使用流
                    Ok(UnifiedFileStream::OssRangeSupported(stream))
                }
                crate::common::OssDownloadResult::FullStreamOnly(stream) => {
                    // 不支持 Range，需要在客户端处理偏移和长度限制
                    Ok(UnifiedFileStream::OssFullStream {
                        stream,
                        skip_bytes: offset,
                        skipped: 0,
                        read_limit: length,
                        read_bytes: 0,
                    })
                }
            }
        }
    }

    // ==================== 内部辅助方法 ====================

    /// 查询 file_local 记录（内部方法）
    async fn get_file_local_record_for_read(&self, file_id: u64) -> FileResult<FileLocalModel> {
        self.helper
            .find_file_local_by_file_id(file_id)
            .await?
            .ok_or_else(|| {
                FileError::System(fluent_message!(
                    "file-error",
                    &format!("File local record not found: {}", file_id)
                ))
            })
    }

    /// 查询 file_oss 记录（内部方法）
    async fn get_file_oss_record(&self, file_id: u64) -> FileResult<FileOssModel> {
        self.helper
            .find_file_oss_by_file_id(file_id)
            .await?
            .ok_or_else(|| {
                FileError::System(fluent_message!(
                    "file-error",
                    &format!("File OSS record not found: {}", file_id)
                ))
            })
    }

    /// 构建文件完整路径（内部方法）
    async fn build_file_path_for_read(
        &self,
        file: &FileModel,
        local_record: &FileLocalModel,
    ) -> FileResult<PathBuf> {
        let base_dir = self.helper.config.get_base_path(&file.storage_type).await?;

        if local_record.local_path.is_empty() {
            return Err(FileError::System(fluent_message!(
                "file-error",
                &format!("Local path is empty for file_id: {}", file.id)
            )));
        }

        Ok(base_dir.join(&local_record.local_path))
    }
}
