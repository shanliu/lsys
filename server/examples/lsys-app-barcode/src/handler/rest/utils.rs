use std::path::PathBuf;

pub(crate) async fn upload_field(
    field: axum::extract::multipart::Field<'_>,
) -> Result<(tempfile::TempDir, PathBuf, String), String> {
    use std::{ffi::OsStr, path::Path, time::SystemTime};
    use tempfile::Builder;
    use tokio::fs::OpenOptions;
    use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};

    let tmp_dir = Builder::new()
        .prefix("barcode")
        .tempdir()
        .map_err(|e| format!("barcode-file-dir-error:{e}"))?;

    let random_number = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let file_path = tmp_dir.path().join(format!("{}.tmp", random_number));

    let mut tmp_file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .await
        .map_err(|e| format!("barcode-file-create-error:{e}"))?;

    let file_name = field.file_name().map(|n| n.to_string());
    let mut ext = file_name
        .as_deref()
        .and_then(|n| Path::new(n).extension().and_then(OsStr::to_str))
        .unwrap_or("")
        .to_string();

    let data = field
        .bytes()
        .await
        .map_err(|e| format!("barcode-file-data-error:{e}"))?;

    tmp_file
        .write_all(&data)
        .await
        .map_err(|e| format!("barcode-file-write-error:{e}"))?;

    if ext != "svg" {
        tmp_file
            .seek(SeekFrom::Start(0))
            .await
            .map_err(|e| format!("barcode-seek-data-error:{e}"))?;
        let mut buffer = [0; 16];
        tmp_file
            .read_exact(&mut buffer)
            .await
            .map_err(|e| format!("barcode-read-data-error:{e}"))?;
        ext = image::guess_format(&buffer)
            .map_err(|e| format!("barcode-format-error:{e}"))?
            .extensions_str()[0]
            .to_string();
    }
    drop(tmp_file);
    Ok((tmp_dir, file_path, ext))
}
