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
    /// 创建用于并发chunk的函数
    ///
    /// 参数: 文件总大小, 并发数量
    /// 使用配置中的 min_chunk_size
    ///
    /// 返回: 适用于创建file_local_chunk记录的数组
    pub fn create_concurrent_chunks(
        &self,
        file_size: u64,
        concurrency: usize,
    ) -> FileResult<Vec<ChunkInfo>> {
        let min_chunk_size = self.config.min_chunk_size;

        if file_size == 0 {
            return Err(FileError::InvalidChunkData(
                "file_size must be greater than 0".to_string(),
            ));
        }

        if concurrency == 0 {
            return Err(FileError::InvalidChunkData(
                "concurrency must be greater than 0".to_string(),
            ));
        }

        let mut chunks = Vec::new();
        let actual_concurrency = (concurrency as u64).min(file_size / min_chunk_size + 1) as usize;

        if actual_concurrency == 0 {
            // 如果文件大小小于最小分片大小，只创建一个chunk
            chunks.push(ChunkInfo {
                offset: 0,
                len: file_size,
                md5: None,
            });
        } else {
            let chunk_size = file_size.div_ceil(actual_concurrency as u64);

            for i in 0..actual_concurrency {
                let offset = (i as u64) * chunk_size;
                let remaining = file_size - offset;
                let len = chunk_size.min(remaining);

                chunks.push(ChunkInfo {
                    offset,
                    len,
                    md5: None,
                });
            }
        }

        Ok(chunks)
    }
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

        let total = validate_chunks(&chunks).unwrap();
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

        let result = validate_chunks(&chunks);
        assert!(result.is_err());
    }

    #[test]
    fn test_validate_chunks_empty() {
        let chunks: Vec<ChunkInfo> = vec![];
        let result = validate_chunks(&chunks);
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

        let total = validate_chunks(&chunks).unwrap();
        assert_eq!(total, 3000);
    }
}
