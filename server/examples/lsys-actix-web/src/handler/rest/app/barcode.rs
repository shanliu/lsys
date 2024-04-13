use std::time::SystemTime;

use crate::common::handler::{ResponseJson, ResponseJsonResult, RestQuery};
use actix_multipart::{Field, Multipart};
use actix_web::post;
use futures_util::{StreamExt, TryStreamExt};
use lsys_app::model::AppsModel;
use lsys_app_barcode::dao::{BarcodeParseRecord, ParseData};
use lsys_core::fluent_message;
use lsys_web::{
    handler::app::{barcode_parse, barcode_show_base64, BarCodeParseParam, BarCodeShowParam},
    JsonData,
};
use serde_json::json;
use tempfile::Builder;
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
async fn parse_field(
    mut field: Field,
    app: &AppsModel,
    param: &BarCodeParseParam,
    rest: &RestQuery,
) -> Result<(String,String,ParseData), String> {
    let tmp_dir = Builder::new()
        .prefix("barcode")
        .tempdir()
        .map_err(|e| rest.fluent_string(fluent_message!("barcode-file-dir-error", e)))?;
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
        .map_err(|e| rest.fluent_string(fluent_message!("barcode-file-create-error", e)))?;
    while let Some(chunk) = field.next().await {
        let data =
            chunk.map_err(|e| rest.fluent_string(fluent_message!("barcode-file-data-error", e)))?;
        tmp_file
            .write_all(&data)
            .await
            .map_err(|e| rest.fluent_string(fluent_message!("barcode-file-write-error", e)))?;
    }

    let mut ext = if let Some(extfile_name) = field.content_disposition().get_filename() {
        extfile_name
            .rfind('.')
            .map(|index| &extfile_name[index + 1..])
            .unwrap_or("")
    } else {
        ""
    };
    dbg!(&file_path);
    if ext != "svg" {
        tmp_file
            .seek(SeekFrom::Start(0))
            .await
            .map_err(|e| rest.fluent_string(fluent_message!("barcode-seek-data-error", e)))?;
        let mut buffer = [0; 16];
        tmp_file
            .read_exact(&mut buffer)
            .await
            .map_err(|e| rest.fluent_string(fluent_message!("barcode-read-data-error", e)))?;
        ext = image::guess_format(&buffer)
            .map_err(|e| rest.fluent_string(fluent_message!("barcode-format-error", e)))?
            .extensions_str()[0];
    }
    drop(tmp_file);
    match barcode_parse(file_path, ext, &app.user_id, &app.id, param, rest)
    .await{
        Ok(tmp)=>{
            match tmp{
                BarcodeParseRecord::Succ((t,record))=>{
                    Ok((t.file_hash,t.barcode_type,record))
                }
                BarcodeParseRecord::Fail(t)=>{
                    Err(rest.fluent_string(fluent_message!("barcode-parse-error", t.record)))
                }
            }
        }
        Err(err)=>Err(rest.fluent_string(err))
    }
}

#[post("barcode")]
pub(crate) async fn barcode(
    mut payload: Multipart,
    mut rest: RestQuery,
) -> ResponseJsonResult<ResponseJson> {
    Ok(match rest.rfc.method.as_deref() {
        Some("parse") => {
            let app = rest.to_app_model().await?;
            let param = rest.param::<BarCodeParseParam>()?;
            dbg!(&param);
            let mut out = vec![];
            while let Ok(Some(field)) = payload.try_next().await {
                out.push(match parse_field(field, &app, &param, &rest).await {
                    Ok((file_hash,btype,record)) => json!({
                        "status":"1",
                        "data":json!({
                            "type":btype,
                            "text":record.text,
                            "position":record.position,
                            "hash":file_hash,
                         })
                    }),
                    Err(err) => json!({
                        "status":"0",
                        "msg":err
                    }),
                });
            }
            Ok(JsonData::data(json!({ "record": out })))
        }
        Some("create") => {
            drop(payload);
            barcode_show_base64(&rest.param::<BarCodeShowParam>()?, &rest).await
        }
        var => handler_not_found!(var.unwrap_or_default()),
    }?
    .into())
}
