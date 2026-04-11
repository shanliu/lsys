use aes::Aes256;
use aes::cipher::{KeyIvInit, StreamCipher};
use ctr::Ctr128BE;
use futures_util::Stream;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

type Aes256Ctr = Ctr128BE<Aes256>;

/// AES-CTR 加密密钥 (32 字节用于 AES-256)
const CRYPTO_KEY: &[u8; 32] = b"change-this-to-your-secret-key!!"; // 正好 32 字节

/// 加密分段大小 (默认 1MB)
const CHUNK_SIZE: usize = 1024 * 1024;

/// 加密块信息
#[derive(Debug, Clone)]
pub struct ChunkBlock {
    /// 块索引
    pub index: usize,
    /// 块在加密文件中的起始位置
    pub file_offset: u64,
    /// 块大小
    pub size: usize,
    /// 在请求数据中的起始位置
    pub data_offset: usize,
    /// 需要从块中读取的起始位置
    pub read_start: usize,
    /// 需要从块中读取的长度
    pub read_len: usize,
}

/// 解密数据异步迭代器
pub struct DecryptIterator {
    /// 加密文件完整路径
    file_path: PathBuf,
    /// 文件总大小
    file_size: u64,
    /// 读取起始偏移
    start_offset: u64,
    /// 读取结束偏移
    end_offset: u64,
    /// 当前块索引
    current_chunk_index: usize,
    /// 起始块索引
    start_chunk_index: usize,
    /// 结束块索引
    end_chunk_index: usize,
    /// 当前数据偏移（在返回数据中的位置）
    current_data_offset: usize,
}

impl DecryptIterator {
    fn new(
        file_path: PathBuf,
        file_size: u64,
        start_offset: u64,
        end_offset: u64,
    ) -> Self {
        let start_chunk = (start_offset / CHUNK_SIZE as u64) as usize;
        let end_chunk = ((end_offset - 1) / CHUNK_SIZE as u64) as usize;
        
        Self {
            file_path,
            file_size,
            start_offset,
            end_offset,
            current_chunk_index: start_chunk,
            start_chunk_index: start_chunk,
            end_chunk_index: end_chunk,
            current_data_offset: 0,
        }
    }

    /// 获取总块数
    pub fn total_chunks(&self) -> usize {
        self.end_chunk_index - self.start_chunk_index + 1
    }

    /// 获取读取范围信息
    pub fn range_info(&self) -> (u64, u64, u64) {
        (self.start_offset, self.end_offset, self.end_offset - self.start_offset)
    }

    /// 异步获取下一个块
    pub async fn next_chunk(&mut self) -> Option<io::Result<(ChunkBlock, Vec<u8>)>> {
        if self.current_chunk_index > self.end_chunk_index {
            return None;
        }

        let chunk_index = self.current_chunk_index;
        let chunk_file_offset = (chunk_index * CHUNK_SIZE) as u64;
        let chunk_end = ((chunk_index + 1) * CHUNK_SIZE) as u64;
        
        // 计算在当前块中的读取范围
        let read_start = if chunk_index == (self.start_offset / CHUNK_SIZE as u64) as usize {
            (self.start_offset % CHUNK_SIZE as u64) as usize
        } else {
            0
        };
        
        let read_end = if chunk_index == ((self.end_offset - 1) / CHUNK_SIZE as u64) as usize {
            ((self.end_offset - 1) % CHUNK_SIZE as u64) as usize + 1
        } else {
            CHUNK_SIZE
        };
        
        let read_len = read_end - read_start;
        
        // 实际块大小（最后一块可能不足 CHUNK_SIZE）
        let actual_chunk_size = if chunk_end > self.file_size {
            (self.file_size - chunk_file_offset) as usize
        } else {
            CHUNK_SIZE
        };
        
        let chunk = ChunkBlock {
            index: chunk_index,
            file_offset: chunk_file_offset,
            size: actual_chunk_size,
            data_offset: self.current_data_offset,
            read_start,
            read_len,
        };
        
        // 更新状态
        self.current_chunk_index += 1;
        self.current_data_offset += read_len;
        
        // 异步读取并解密块数据
        let result = decrypt_chunk(&self.file_path, &chunk).await;
        Some(result.map(|data| (chunk, data)))
    }
}

impl Stream for DecryptIterator {
    type Item = io::Result<(ChunkBlock, Vec<u8>)>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // 创建异步任务
        let fut = self.next_chunk();
        tokio::pin!(fut);
        
        match fut.poll(cx) {
            Poll::Ready(result) => Poll::Ready(result),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// 加密文件
/// 
/// # Arguments
/// * `source_path` - 待加密文件路径
/// * `dest_path` - 目标加密文件的完整路径
/// 
/// # Returns
/// * `Ok(())` - 加密成功
/// * `Err(io::Error)` - 加密失败
pub async fn encrypt_file(
    source_path: impl AsRef<Path>,
    dest_path: impl AsRef<Path>,
) -> io::Result<()> {
    let source_path = source_path.as_ref();
    let dest_path = dest_path.as_ref();
    
    // 确保目标目录存在
    if let Some(parent) = dest_path.parent() {
        create_dir_all(parent).await?;
    }
    
    // 打开源文件和目标文件
    let mut source_file = File::open(source_path).await?;
    let mut dest_file = File::create(dest_path).await?;
    
    // 分块加密
    let mut chunk_index = 0u64;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    
    loop {
        let bytes_read = source_file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        
        // 为每个块创建独立的 IV（使用块索引）
        let mut iv = [0u8; 16];
        iv[..8].copy_from_slice(&chunk_index.to_be_bytes());
        
        // 创建加密器
        let mut cipher = Aes256Ctr::new(CRYPTO_KEY.into(), &iv.into());
        
        // 加密数据
        let mut encrypted_data = buffer[..bytes_read].to_vec();
        cipher.apply_keystream(&mut encrypted_data);
        
        // 写入加密数据
        dest_file.write_all(&encrypted_data).await?;
        
        chunk_index += 1;
    }
    
    dest_file.flush().await?;
    
    Ok(())
}

/// 解密文件片段（创建异步迭代器）
/// 
/// # Arguments
/// * `encrypted_path` - 加密文件的完整路径
/// * `offset` - 读取起始位置（相对于原始文件）
/// * `length` - 读取长度，None 表示读取到文件末尾
/// 
/// # Returns
/// * `Ok(DecryptIterator)` - 解密异步迭代器（惰性求值，每次迭代时才解密对应块）
/// * `Err(io::Error)` - 创建失败
pub async fn decrypt_file_range(
    encrypted_path: impl AsRef<Path>,
    offset: u64,
    length: Option<u64>,
) -> io::Result<DecryptIterator> {
    let encrypted_path = encrypted_path.as_ref();
    
    // 获取文件大小
    let metadata = tokio::fs::metadata(encrypted_path).await?;
    let file_size = metadata.len();
    
    // 计算实际读取长度
    let actual_length = length.unwrap_or(file_size.saturating_sub(offset));
    let end_offset = offset + actual_length;
    
    if end_offset > file_size {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Read range exceeds file size",
        ));
    }
    
    if actual_length == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Read length cannot be zero",
        ));
    }
    
    Ok(DecryptIterator::new(encrypted_path.to_path_buf(), file_size, offset, end_offset))
}

/// 异步解密单个块
async fn decrypt_chunk(file_path: &Path, chunk: &ChunkBlock) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path).await?;
    
    // 定位到块位置
    file.seek(tokio::io::SeekFrom::Start(chunk.file_offset)).await?;
    
    // 读取加密数据
    let mut encrypted_data = vec![0u8; chunk.size];
    file.read_exact(&mut encrypted_data).await?;
    
    // 为该块创建 IV
    let mut iv = [0u8; 16];
    iv[..8].copy_from_slice(&(chunk.index as u64).to_be_bytes());
    
    // 创建解密器
    let mut cipher = Aes256Ctr::new(CRYPTO_KEY.into(), &iv.into());
    
    // 解密数据
    cipher.apply_keystream(&mut encrypted_data);
    
    // 返回需要的部分
    Ok(encrypted_data[chunk.read_start..chunk.read_start + chunk.read_len].to_vec())
}

/// 获取加密文件的原始大小（解密后的大小）
/// 
/// # Arguments
/// * `encrypted_path` - 加密文件的完整路径
/// 
/// # Returns
/// * `Ok(u64)` - 原始文件大小
/// * `Err(io::Error)` - 获取失败
pub async fn get_decrypted_size(
    encrypted_path: impl AsRef<Path>,
) -> io::Result<u64> {
    let metadata = tokio::fs::metadata(encrypted_path.as_ref()).await?;
    Ok(metadata.len())
}

/// 验证加密文件是否可以正常解密（读取第一个块）
/// 
/// # Arguments
/// * `encrypted_path` - 加密文件的完整路径
/// 
/// # Returns
/// * `Ok(true)` - 文件可以正常解密
/// * `Ok(false)` - 文件无法解密
/// * `Err(io::Error)` - 验证过程出错
pub async fn verify_encrypted_file(
    encrypted_path: impl AsRef<Path>,
) -> io::Result<bool> {
    let encrypted_path = encrypted_path.as_ref();
    
    // 检查文件是否存在
    if !tokio::fs::try_exists(encrypted_path).await? {
        return Ok(false);
    }
    
    let metadata = tokio::fs::metadata(encrypted_path).await?;
    let file_size = metadata.len();
    
    if file_size == 0 {
        return Ok(true); // 空文件也是有效的
    }
    
    // 尝试读取第一个块
    let read_size = std::cmp::min(CHUNK_SIZE as u64, file_size);
    let chunk = ChunkBlock {
        index: 0,
        file_offset: 0,
        size: read_size as usize,
        data_offset: 0,
        read_start: 0,
        read_len: read_size as usize,
    };
    
    match decrypt_chunk(encrypted_path, &chunk).await {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// 解密文件到指定路径
/// 
/// # Arguments
/// * `encrypted_path` - 加密文件的完整路径
/// * `target_path` - 目标文件的完整路径
/// 
/// # Returns
/// * `Ok(())` - 解密成功
/// * `Err(io::Error)` - 解密失败
pub async fn decrypt_file_to(
    encrypted_path: impl AsRef<Path>,
    target_path: impl AsRef<Path>,
) -> io::Result<()> {
    let target_path = target_path.as_ref();
    
    // 确保目标目录存在
    if let Some(parent) = target_path.parent() {
        create_dir_all(parent).await?;
    }
    
    // 创建解密迭代器
    let mut iter = decrypt_file_range(
        encrypted_path,
        0,
        None,
    ).await?;
    
    // 打开目标文件
    let mut dest_file = File::create(target_path).await?;
    
    // 逐块解密并写入
    while let Some(result) = iter.next_chunk().await {
        let (_, data) = result?;
        dest_file.write_all(&data).await?;
    }
    
    dest_file.flush().await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_encrypt_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test_encrypted.enc");
        
        // 创建测试文件
        let test_data = b"Hello, World! This is a test file for encryption.";
        tokio::fs::write(&source_path, test_data).await.unwrap();
        
        // 加密
        encrypt_file(&source_path, &encrypted_path).await.unwrap();
        assert!(tokio::fs::try_exists(&encrypted_path).await.unwrap());
        
        // 解密全部内容
        let mut decrypted_data = Vec::new();
        let mut iter = decrypt_file_range(&encrypted_path, 0, None).await.unwrap();
        
        while let Some(result) = iter.next_chunk().await {
            let (_, data) = result.unwrap();
            decrypted_data.extend_from_slice(&data);
        }
        
        assert_eq!(&decrypted_data, test_data);
    }

    #[tokio::test]
    async fn test_partial_decrypt() {
        let temp_dir = TempDir::new().unwrap();
        let source_path = temp_dir.path().join("test.txt");
        let encrypted_path = temp_dir.path().join("test_partial.enc");
        
        // 创建测试文件
        let test_data = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        tokio::fs::write(&source_path, test_data).await.unwrap();
        
        // 加密
        encrypt_file(&source_path, &encrypted_path).await.unwrap();
        
        // 解密部分内容 (offset=10, length=10)
        let mut decrypted_data = Vec::new();
        let mut iter = decrypt_file_range(&encrypted_path, 10, Some(10)).await.unwrap();
        
        while let Some(result) = iter.next_chunk().await {
            let (_, data) = result.unwrap();
            decrypted_data.extend_from_slice(&data);
        }
        
        assert_eq!(&decrypted_data, b"ABCDEFGHIJ");
    }
}
