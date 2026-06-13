use actix_web::{HttpRequest, HttpResponse};

/// 解析 HTTP Range 请求头，返回起始字节偏移量
///
/// 支持格式：
/// - `bytes=500-`   → `Some(500)`（开放结束范围）
/// - `bytes=0-1023` → `Some(0)`（固定范围，仅取 start）
/// - `bytes=-500`   → `None`（后缀范围，不支持，调用方应返回完整内容）
/// - 多段范围 `bytes=0-100,200-300` → `Some(0)`（仅取第一段 start，忽略后续段）
/// - 无 Range 头或格式无法识别 → `None`
pub fn parse_range_header(req: &HttpRequest) -> Option<u64> {
    let range_header = req.headers().get("Range")?;
    let range_str = range_header.to_str().ok()?;
    // 只处理 bytes= 格式
    let bytes_part = range_str.trim().strip_prefix("bytes=")?;
    // 取第一段（多段范围只用第一段）
    let first_range = bytes_part.split(',').next()?.trim();
    // 后缀范围（如 bytes=-500）不支持，返回 None
    if first_range.starts_with('-') {
        return None;
    }
    // 取 start 部分（'-' 之前）
    let start_str = first_range.split('-').next()?.trim();
    start_str.parse::<u64>().ok()
}

/// 创建文件流式响应
///
/// 将文件流转换为 HTTP 流式响应，支持断点续传
///
/// # Parameters
///
/// - `ref_model`: 文件引用模型
/// - `file_model`: 文件模型
/// - `stream`: 文件流
/// - `offset`: 起始偏移量（用于断点续传）
///
/// # Returns
///
/// 返回配置好的 HttpResponse，包含正确的 Content-Type、Content-Disposition 和 Range 头
pub fn create_file_stream_response(
    ref_model: lsys_web::lsys_file::model::FileRefModel,
    file_model: lsys_web::lsys_file::model::FileModel,
    mut stream: lsys_web::lsys_file::dao::UnifiedFileStream,
    offset: u64,
) -> HttpResponse {
    // 创建异步流
    let stream = async_stream::stream! {
        while let Some(result) = stream.next_bytes().await {
            match result {
                Ok(bytes) => yield Ok::<actix_web::web::Bytes, std::io::Error>(bytes),
                Err(e) => {
                    tracing::error!("Error reading file stream: {:?}", e);
                    break;
                }
            }
        }
    };

    let mut response = HttpResponse::Ok();
    response.content_type(file_model.content_type.as_str());
    response.insert_header((
        "Content-Disposition",
        format!("inline; filename=\"{}\"", ref_model.file_name),
    ));

    if offset == 0 {
        response.insert_header(("Content-Length", file_model.file_size.to_string()));
    } else {
        response.status(actix_web::http::StatusCode::PARTIAL_CONTENT);
        response.insert_header((
            "Content-Range",
            format!(
                "bytes {}-{}/{}",
                offset,
                file_model.file_size - 1,
                file_model.file_size
            ),
        ));
    }

    response.streaming(Box::pin(stream))
}

/// 创建文件下载响应（用于导出任务等场景）
///
/// 将流转换为 HTTP 下载响应，支持断点续传和 UTF-8 文件名
///
/// # Parameters
///
/// - `file_name`: 文件名
/// - `content_type`: 内容类型
/// - `file_size`: 文件大小
/// - `stream`: 文件流（任何实现了 Stream 的类型）
/// - `offset`: 起始偏移量（用于断点续传）
///
/// # Returns
///
/// 返回配置好的 HttpResponse，包含 attachment 下载头和 UTF-8 文件名编码
/// 根据 MIME 类型推断文件扩展名（不含点）
fn mime_to_ext(content_type: &str) -> Option<&'static str> {
    let mime = content_type.split(';').next().unwrap_or("").trim();
    mime_guess::get_mime_extensions_str(mime)
        .and_then(|exts| exts.first())
        .copied()
}

pub fn create_download_response<S>(
    file_name: String,
    content_type: String,
    file_size: u64,
    stream: S,
    offset: u64,
) -> HttpResponse
where
    S: futures_util::Stream<Item = Result<actix_web::web::Bytes, std::io::Error>> + 'static,
{
    // 若文件名没有扩展名，根据 content_type 补充
    let file_name = {
        let has_ext = std::path::Path::new(&file_name).extension().is_some();
        if !has_ext {
            if let Some(ext) = mime_to_ext(&content_type) {
                format!("{}.{}", file_name, ext)
            } else {
                file_name
            }
        } else {
            file_name
        }
    };

    let mut response = HttpResponse::Ok();

    // 设置 Content-Type
    response.content_type(content_type);

    // 设置 Content-Disposition（attachment 表示下载，支持 UTF-8 文件名）
    let encoded_filename = urlencoding::encode(&file_name);
    response.insert_header((
        "Content-Disposition",
        format!(
            "attachment; filename=\"{}\"; filename*=UTF-8''{}",
            file_name, encoded_filename
        ),
    ));

    // 如果是 Range 请求，设置相应的响应头
    if offset > 0 {
        response.status(actix_web::http::StatusCode::PARTIAL_CONTENT);
        response.insert_header((
            "Content-Range",
            format!("bytes {}-{}/{}", offset, file_size - 1, file_size),
        ));
        response.insert_header(("Content-Length", (file_size - offset).to_string()));
    } else {
        response.insert_header(("Content-Length", file_size.to_string()));
    }

    // 设置 Accept-Ranges 支持断点续传
    response.insert_header(("Accept-Ranges", "bytes"));

    response.streaming(Box::pin(stream))
}
