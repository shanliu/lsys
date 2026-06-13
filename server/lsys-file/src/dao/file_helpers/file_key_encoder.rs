use crate::common::FileError;
use sqids::Sqids;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct FileKeyEncoder {
    sqids: Sqids,
}

impl FileKeyEncoder {
    pub fn new(salt: &str, min_length: u8) -> Self {
        // 使用 salt 简单地置换一下默认字母表，使得每个项目的加密结果不同
        let default_alphabet = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
        let mut chars: Vec<char> = default_alphabet.chars().collect();
        let salt_bytes = salt.as_bytes();
        let salt_len = salt_bytes.len();

        if salt_len > 0 {
            for i in 0..chars.len() {
                let swap_idx = (i + salt_bytes[i % salt_len] as usize) % chars.len();
                chars.swap(i, swap_idx);
            }
        }

        let sqids = Sqids::builder()
            .alphabet(chars)
            .min_length(min_length)
            .build()
            .unwrap_or_else(|_| Sqids::default());
        Self { sqids }
    }

    /// u64 转 字符串 (支持可选的过期时间戳)
    ///
    /// expire_time: 绝对时间戳（秒），None 表示不过期。
    pub fn encode(&self, id: u64, expire_time: Option<u64>) -> String {
        // 固定编码两个值 [id, expire_time]。
        // 如果不过期，expire_time = 0。这样可以保证生成的字符串长度更统一。
        let exp = expire_time.unwrap_or(0);
        self.sqids.encode(&[id, exp]).unwrap_or_default()
    }

    /// 字符串 转 u64，校验过期时间
    pub fn decode(&self, key: &str) -> Result<u64, FileError> {
        let numbers = self.sqids.decode(key);
        if numbers.is_empty() {
            return Err(FileError::InvalidFileKey(key.to_string()));
        }

        let id = numbers[0];

        // 校验过期时间
        if numbers.len() > 1 {
            let exp = numbers[1];
            if exp > 0 {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();
                if now > exp {
                    return Err(FileError::FileKeyExpired(key.to_string()));
                }
            }
        }

        Ok(id)
    }
}
