use crate::common::{FileError, FileResult};

/// 并发chunk信息结构体
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    /// 文件偏移量（字节）
    pub offset: u64,
    /// 分片大小（字节）
    pub len: u64,
    /// MD5哈希值（可选，创建时为None）
    pub md5: Option<String>,
}

impl super::FileHelper {
    /// 按总大小拆分下载分片
    ///
    /// 参数: 文件总大小
    /// 使用配置中的 download_chunk_max：分片数 = ceil(总大小 / download_chunk_max)，再对总大小平均拆分
    ///
    /// 返回: 适用于创建file_local_chunk记录的数组
    pub fn create_download_chunks(&self, file_size: u64) -> FileResult<Vec<ChunkInfo>> {
        if file_size == 0 {
            return Err(FileError::InvalidChunkData(
                "file_size must be greater than 0".to_string(),
            ));
        }

        let chunk_max = self.config.download_chunk_max.max(1);
        let num_chunks = file_size.div_ceil(chunk_max).max(1);
        let chunk_size = file_size.div_ceil(num_chunks);

        let mut chunks = Vec::new();
        let mut offset = 0u64;
        while offset < file_size {
            let len = chunk_size.min(file_size - offset);
            chunks.push(ChunkInfo {
                offset,
                len,
                md5: None,
            });
            offset += len;
        }

        Ok(chunks)
    }

    /// 检查并发chunk的数据是否合规
    ///
    /// 参数: chunk数据数组
    ///
    /// 返回: 总大小或错误
    pub fn validate_chunks(chunks: &[ChunkInfo]) -> FileResult<u64> {
        if chunks.is_empty() {
            return Err(FileError::InvalidChunkData(
                "chunks array is empty".to_string(),
            ));
        }

        // 按offset排序检查
        let mut sorted_chunks = chunks.to_vec();
        sorted_chunks.sort_by_key(|c| c.offset);

        let mut expected_offset = 0u64;

        for chunk in sorted_chunks.iter() {
            // 检查offset + len 是否连续
            if chunk.offset != expected_offset {
                return Err(FileError::InvalidChunkData(format!(
                    "gap found at offset: expected {}, got {}",
                    expected_offset, chunk.offset
                )));
            }

            expected_offset = chunk.offset + chunk.len;
        }

        Ok(expected_offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_chunks_valid() {
        let chunks = vec![
            ChunkInfo {
                offset: 0,
                len: 1000,
                md5: None,
            },
            ChunkInfo {
                offset: 1000,
                len: 2000,
                md5: None,
            },
            ChunkInfo {
                offset: 3000,
                len: 1500,
                md5: None,
            },
        ];

        let total = super::super::FileHelper::validate_chunks(&chunks).unwrap();
        assert_eq!(total, 4500);
    }

    #[test]
    fn test_validate_chunks_gap() {
        let chunks = vec![
            ChunkInfo {
                offset: 0,
                len: 1000,
                md5: None,
            },
            ChunkInfo {
                offset: 1100, // gap: should be 1000
                len: 2000,
                md5: None,
            },
        ];

        let result = super::super::FileHelper::validate_chunks(&chunks);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chunks_empty() {
        let chunks: Vec<ChunkInfo> = vec![];
        let result = super::super::FileHelper::validate_chunks(&chunks);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chunks_unordered() {
        let chunks = vec![
            ChunkInfo {
                offset: 2000,
                len: 1000,
                md5: None,
            },
            ChunkInfo {
                offset: 0,
                len: 2000,
                md5: None,
            },
        ];

        let total = super::super::FileHelper::validate_chunks(&chunks).unwrap();
        assert_eq!(total, 3000);
    }
}
