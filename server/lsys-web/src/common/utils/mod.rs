/// 判断字节数据是否为图片类型（基于 magic bytes）。
///
/// # 支持的格式
/// jpeg / png / gif / webp / bmp / tiff / ico / avif / heic / heif
///
/// # 参数
/// - `data`: 文件前几十字节，建议至少 16 字节
///
/// # 返回
/// - `Some(&str)` — 识别到的图片 MIME type
/// - `None` — 无法识别，或不是图片类型
pub fn get_image_mime(data: &[u8]) -> Option<&'static str> {
    let kind = infer::get(data)?;
    let mime = kind.mime_type();
    if matches!(
        mime,
        "image/jpeg"
            | "image/png"
            | "image/gif"
            | "image/webp"
            | "image/bmp"
            | "image/tiff"
            | "image/x-icon"
            | "image/avif"
            | "image/heic"
            | "image/heif"
    ) {
        Some(mime)
    } else {
        None
    }
}


