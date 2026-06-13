pub mod crypto;
mod error;
mod file;
mod oss;
mod utils;

// Re-export error types
pub use error::{FileError, FileResult};

// Re-export oss types
pub use oss::{
    OssDownloadChunk, OssDownloadResult, OssDownloadStream, OssObjectMeta, OssProvider,
    OssProviderConfig, OssResult, UploadFileInfo,
};

// Re-export utils
pub(crate) use utils::{extract_extension, rand_simple, sanitize_filename};

// Re-export crypto types (public for dao usage)
pub use crypto::{
    DecryptIterator, decrypt_file_range, decrypt_file_to, encrypt_file, get_decrypted_size,
    verify_encrypted_file,
};

// Re-export file types
pub(crate) use file::get_content_type;
