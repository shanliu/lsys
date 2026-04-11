use super::FileResult;
use tokio::io::AsyncReadExt;
use tracing::{debug, warn};

/// 获取文件 content_type，传入完整文件路径（异步）
///
/// 基于文件内容（文件头）进行嗅探来返回 MIME 类型，依赖 `infer`。
pub async fn get_content_type(file_path: impl AsRef<std::path::Path>) -> FileResult<String> {
    let path = file_path.as_ref();

    let path_str = path.to_string_lossy();
    debug!("Start inferring file MIME type: {}", path_str);

    // 只读取前 N 字节用于嗅探，避免读取整个大文件
    const READ_BYTES: usize = 8192;
    let mut f = tokio::fs::File::open(path).await?;
    let mut buf = vec![0u8; READ_BYTES];
    let n = f.read(&mut buf).await?;
    buf.truncate(n);

    let mime_type = match infer::get(&buf) {
        Some(kind) => {
            let mt = kind.mime_type().to_string();
            debug!("Inferred MIME type for {}: {}", path_str, mt);
            mt
        }
        None => {
            // 如果基于文件头无法识别，则尝试基于扩展名回退识别
            if let Some(mt) = mime_guess::from_path(path).first().map(|m| m.to_string()) {
                debug!(
                    "Fallback to extension-based MIME type for {}: {}",
                    path_str, mt
                );
                mt
            } else {
                warn!(
                    "Unable to infer MIME type for {} using content sniffing or extension",
                    path_str
                );
                "application/octet-stream".to_string()
            }
        }
    };

    debug!("File {} MIME type: {}", path_str, mime_type);

    Ok(mime_type)
}
