use super::FileResult;

/// 获取文件 content_type，传入完整文件路径
///
/// 根据文件路径返回文件mime类型
pub fn get_content_type(file_path: &str) -> FileResult<String> {
    let path = std::path::Path::new(file_path);

    // 使用 mime_guess 根据文件扩展名猜测 MIME 类型
    let mime_type = mime_guess::from_path(path)
        .first()
        .map(|m| m.to_string())
        .unwrap_or_else(|| "application/octet-stream".to_string());

    Ok(mime_type)
}
