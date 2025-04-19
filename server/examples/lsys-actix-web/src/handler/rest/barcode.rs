use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
    time::SystemTime,
};

use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_multipart::{Field, Multipart};
use actix_web::post;
use futures_util::{StreamExt, TryStreamExt};
use lsys_core::fluent_message;
use lsys_web::{
    common::{JsonData, JsonError, JsonResponse},
    handler::rest::barcode::{barcode_base64, parse_image, CodeParam, ParseParam},
};
use serde_json::json;
use tempfile::Builder;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};

#[post("")]
pub(crate) async fn barcode(
    mut payload: Multipart,
    rest: RestQuery,
) -> ResponseJsonResult<ResponseJson> {
    let app = rest.get_app().await?;
    Ok(match rest.rfc.method.as_deref().unwrap_or_default() {
        "parse" => {
            let param = rest.param::<ParseParam>()?;
            dbg!(&param);
            let mut out = vec![];
            while let Ok(Some(field)) = payload.try_next().await {
                out.push(match upload_file(field, &rest).await {
                    Ok((file_path, ext)) => {
                        match parse_image(file_path, &ext, &param, &app, &rest).await {
                            Ok(dat) => {
                                json!({
                                    "status":"1",
                                    "data":dat
                                })
                            }
                            Err(err) => json!({
                                "status":"0",
                                "msg": rest.fluent_error_string(&err)
                            }),
                        }
                    }
                    Err(err) => json!({
                        "status":"0",
                        "msg":err
                    }),
                });
            }
            Ok(JsonResponse::data(JsonData::body(json!({ "record": out }))))
        }
        "create" => {
            drop(payload);
            barcode_base64(&rest.param::<CodeParam>()?, &app, &rest).await
        }
        var => handler_not_found!(var),
    }
    .map_err(|e| rest.fluent_error_json_response(&e))?
    .into())
}

async fn upload_file(mut field: Field, rest: &RestQuery) -> Result<(PathBuf, String), String> {
    let tmp_dir = Builder::new().prefix("barcode").tempdir().map_err(|e| {
        rest.fluent_error_string(&JsonError::Message(fluent_message!(
            "barcode-file-dir-error",
            e
        )))
    })?;
    let random_number = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let file_path = tmp_dir.path().join(format!("{}.tmp", random_number));
    let mut tmp_file = tokio::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(true)
        .open(&file_path)
        .await
        .map_err(|e| {
            rest.fluent_error_string(&JsonError::Message(fluent_message!(
                "barcode-file-create-error",
                e
            )))
        })?;
    while let Some(chunk) = field.next().await {
        let data = chunk.map_err(|e| {
            rest.fluent_error_string(&JsonError::Message(fluent_message!(
                "barcode-file-data-error",
                e
            )))
        })?;
        tmp_file.write_all(&data).await.map_err(|e| {
            rest.fluent_error_string(&JsonError::Message(fluent_message!(
                "barcode-file-write-error",
                e
            )))
        })?;
    }

    let mut ext = if let Some(extfile_name) = field.content_disposition() {
        if let Some(file_name) = extfile_name.get_filename() {
            Path::new(file_name)
                .extension()
                .and_then(OsStr::to_str)
                .unwrap_or("")
        } else {
            ""
        }
    } else {
        ""
    };
    dbg!(&file_path);
    if ext != "svg" {
        tmp_file.seek(SeekFrom::Start(0)).await.map_err(|e| {
            rest.fluent_error_string(&JsonError::Message(fluent_message!(
                "barcode-seek-data-error",
                e
            )))
        })?;
        let mut buffer = [0; 16];
        tmp_file.read_exact(&mut buffer).await.map_err(|e| {
            rest.fluent_error_string(&JsonError::Message(fluent_message!(
                "barcode-read-data-error",
                e
            )))
        })?;
        ext = image::guess_format(&buffer)
            .map_err(|e| {
                rest.fluent_error_string(&JsonError::Message(fluent_message!(
                    "barcode-format-error",
                    e
                )))
            })?
            .extensions_str()[0];
    }
    drop(tmp_file);
    Ok((file_path, ext.to_string()))
}
