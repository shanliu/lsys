use lsys_logger::dao::ChangeLogData;
use serde::Serialize;

// 文件系统操作日志

/// 文件创建操作日志
#[derive(Serialize)]
pub(crate) struct LogFileCreate<'t> {
    pub action: &'t str,
    pub storage_type: &'t str,
    pub user_id: u64,
    pub file_id: u64,
    pub file_md5: &'t str,
}

impl ChangeLogData for LogFileCreate<'_> {
    fn log_type() -> &'static str {
        "file-create"
    }
    fn message(&self) -> String {
        format!(
            "{} file {} type:{}",
            self.action, self.file_id, self.storage_type
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

/// 文件上传操作日志
#[derive(Serialize)]
pub(crate) struct LogFileUpload<'t> {
    pub action: &'t str,
    pub user_id: u64,
    pub file_id: u64,
    pub file_name: &'t str,
    pub chunk_count: usize,
}

impl ChangeLogData for LogFileUpload<'_> {
    fn log_type() -> &'static str {
        "file-upload"
    }
    fn message(&self) -> String {
        format!(
            "{} upload file:{} name:{}",
            self.action, self.file_id, self.file_name
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

/// 文件删除操作日志
#[derive(Serialize)]
pub(crate) struct LogFileDelete {
    pub user_id: u64,
    pub file_id: u64,
}

impl ChangeLogData for LogFileDelete {
    fn log_type() -> &'static str {
        "file-delete"
    }
    fn message(&self) -> String {
        format!("delete file:{}", self.file_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

/// 文件拷贝操作日志
#[derive(Serialize)]
pub(crate) struct LogFileCopy {
    pub user_id: u64,
    pub source_file_id: u64,
    pub new_file_id: u64,
}

impl ChangeLogData for LogFileCopy {
    fn log_type() -> &'static str {
        "file-copy"
    }
    fn message(&self) -> String {
        format!("copy file:{} to:{}", self.source_file_id, self.new_file_id)
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

/// 文件 OSS 同步操作日志
#[derive(Serialize)]
pub(crate) struct LogFileSync<'t> {
    pub action: &'t str,
    pub user_id: u64,
    pub file_id: u64,
    pub storage_type: &'t str,
}

impl ChangeLogData for LogFileSync<'_> {
    fn log_type() -> &'static str {
        "file-sync"
    }
    fn message(&self) -> String {
        format!(
            "{} file:{} type:{}",
            self.action, self.file_id, self.storage_type
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}

/// 文件过期时间更新操作日志
#[derive(Serialize)]
pub(crate) struct LogFileExpireTimeUpdate {
    pub file_ref_id: u64,
    pub expire_time: u64,
    pub change_user_id: u64,
}

impl ChangeLogData for LogFileExpireTimeUpdate {
    fn log_type() -> &'static str {
        "file-expire-time-update"
    }
    fn message(&self) -> String {
        format!(
            "update file_ref:{} expire_time:{}",
            self.file_ref_id, self.expire_time
        )
    }
    fn encode(&self) -> String {
        serde_json::to_string(&self).unwrap_or_default()
    }
}
