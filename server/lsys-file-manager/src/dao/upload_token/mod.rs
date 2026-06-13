//! 文件上传令牌管理
//!
//! 对齐 S3/OSS 分片上传机制，支持两类场景：
//!
//! ## 单文件上传（小文件）
//! 签发一个**短时令牌**（`UploadToken`），客户端持令牌直接上传，上传完成后销毁。
//! 使用 [`UploadTokenManager::create_token`] / [`UploadTokenManager::resolve_token`] / [`UploadTokenManager::remove_token`]。
//!
//! ## 分片上传（大文件）
//! 1. [`UploadTokenManager::create_session`] 创建**长时会话**（对应 S3 UploadId，默认 12 h）。
//! 2. [`UploadTokenManager::create_part_token`] 为每个分片签发**短时令牌**（默认 30 min），TTL 不超过会话剩余时间。
//! 3. 上传完成后由上层调用业务完成逻辑；中止时调用 [`UploadTokenManager::remove_session`] 批量清理。
//!
//! ## Redis 键设计
//! | 键 | 值 | 说明 |
//! |---|---|---|
//! | `file:upload:token:{token}` | `UploadTokenData` JSON | 短时令牌（单文件或分片） |
//! | `file:upload:session:{session_id}` | `UploadSessionData` JSON | 长时会话（分片上传） |
//! | `file:upload:session:parts:{session_id}` | Redis Set（token 字符串） | 会话下所有分片令牌，批量清理用 |
//! | `file:upload:ref:{file_ref_id}` | `"token:{t}"` 或 `"session:{s}"` | 反向索引，保证每个上传任务最多一个活跃凭证 |

mod internal;
mod session;
mod token;
pub mod types;

pub use types::{UploadSessionData, UploadTokenData};

// ── 有效期常量（pub 供上层参考） ──────────────────────────────────────────────

/// 短时令牌默认有效期（秒）：30 分钟
pub const TOKEN_DEFAULT_EXPIRE_SECS: u64 = 1800;
/// 长时会话默认有效期（秒）：12 小时
pub const SESSION_DEFAULT_EXPIRE_SECS: u64 = 43200;

// ── Redis 键前缀（模块内可见） ────────────────────────────────────────────────

const TOKEN_PREFIX: &str = "file:upload:token:";
const SESSION_PREFIX: &str = "file:upload:session:";
const SESSION_PARTS_PREFIX: &str = "file:upload:session:parts:";
const REF_PREFIX: &str = "file:upload:ref:";

const ID_LEN: usize = 64;

// ── 管理器 ────────────────────────────────────────────────────────────────────

/// 上传令牌管理器（[`Clone`] 友好，内部持 `deadpool_redis::Pool`）
#[derive(Clone)]
pub struct UploadTokenManager {
    pub(super) redis: deadpool_redis::Pool,
}

impl UploadTokenManager {
    pub fn new(redis: deadpool_redis::Pool) -> Self {
        Self { redis }
    }

    // ── 键构造（所有子模块共用） ──────────────────────────────────────────────

    pub(super) fn token_key(token: &str) -> String {
        format!("{TOKEN_PREFIX}{token}")
    }

    pub(super) fn session_key(session_id: &str) -> String {
        format!("{SESSION_PREFIX}{session_id}")
    }

    pub(super) fn session_parts_key(session_id: &str) -> String {
        format!("{SESSION_PARTS_PREFIX}{session_id}")
    }

    pub(super) fn ref_key(file_ref_id: u64) -> String {
        format!("{REF_PREFIX}{file_ref_id}")
    }

    pub(super) fn ref_value_token(token: &str) -> String {
        format!("token:{token}")
    }

    pub(super) fn ref_value_session(session_id: &str) -> String {
        format!("session:{session_id}")
    }

    pub(super) fn gen_id() -> String {
        lsys_core::utils::rand_str(lsys_core::utils::RandType::LowerHex, ID_LEN)
    }
}

