use serde::{Deserialize, Serialize};

/// 短时令牌绑定数据
///
/// 单文件上传时 `session_id` / `part_number` 均为 `None`；
/// 分片上传时两者均有值。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadTokenData {
    /// 上传任务引用 ID（lst_file_ref.id）
    pub file_ref_id: u64,
    /// 上传归属用户
    pub user_id: u64,
    /// 上传归属应用（rest 场景为应用 ID，service 场景恒为 0）
    pub app_id: u64,
    /// 所属分片会话 ID（仅分片上传有值）
    pub session_id: Option<String>,
    /// 分片序号，1-based（仅分片上传有值）
    pub part_number: Option<u32>,
}

/// 长时分片上传会话数据（对应 S3 UploadId）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadSessionData {
    /// 上传任务引用 ID（lst_file_ref.id）
    pub file_ref_id: u64,
    /// 上传归属用户
    pub user_id: u64,
    /// 上传归属应用（rest 场景为应用 ID，service 场景恒为 0）
    pub app_id: u64,
    /// 预期分片总数（用于上层校验完整性）
    pub total_parts: u32,
}
