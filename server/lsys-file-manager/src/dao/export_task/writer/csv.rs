// CSV 文件写入辅助结构
//
// 用法示例：
// ```ignore
// let mut w = CsvWriter::new(&record)
//     .header(("user_id", "name", "email"))
//     .await?;                                        // -> CsvRowWriter<Cols3>
//
// w.write_row((1u64, "alice", "a@b.com")).await?;    // 3-元组 ✓
// w.write_row((1u64, "bob")).await?;                 // 2-元组 ✗ 编译报错
// w.write_batch(vec![(2u64, "bob", "b@b.com")]).await?;
// let path = w.finish().await?;
// ```
//
// 原理：
//   `CsvWriter::header<H: CsvRecord>` 由 H 的元组类型推断出 `H::Arity`（Cols1..Cols12），
//   返回的 `CsvRowWriter<H::Arity>` 只接受相同 Arity 的行，不匹配时编译期报错。

use std::marker::PhantomData;
use std::path::PathBuf;

use tokio::io::AsyncWriteExt;

use crate::dao::FileManagerError;
use crate::model::ExportTaskModel;

// ── 列数 Marker 类型（空结构，仅用于类型层面携带列数信息）────────────────────
pub struct Cols1;
pub struct Cols2;
pub struct Cols3;
pub struct Cols4;
pub struct Cols5;
pub struct Cols6;
pub struct Cols7;
pub struct Cols8;
pub struct Cols9;
pub struct Cols10;
pub struct Cols11;
pub struct Cols12;

// ── CsvRecord trait ──────────────────────────────────────────────────────────
/// 可以作为 CSV 行的类型（由 header 元组 / 数据元组实现）。
///
/// `Arity` 是列数 Marker，`write_row` 要求行与 header 共享同一 `Arity`，
/// 列数不匹配时编译直接报错，无需运行时检查。
pub trait CsvRecord {
    /// 列数 Marker 类型，与 `CsvWriter::header` 保持一致才能 `write_row`
    type Arity;
    fn to_fields(&self) -> Vec<String>;
}

// ── 宏批量为 1-元组 ~ 12-元组 实现 CsvRecord ─────────────────────────────────
macro_rules! impl_csv_record {
    ( $arity:ident ; $( $T:ident : $idx:tt ),+ ) => {
        impl<$($T: std::fmt::Display),+> CsvRecord for ($($T,)+) {
            type Arity = $arity;
            fn to_fields(&self) -> Vec<String> {
                vec![$( csv_escape(&self.$idx.to_string()) ),+]
            }
        }
    };
}

impl_csv_record!(Cols1;  A:0);
impl_csv_record!(Cols2;  A:0, B:1);
impl_csv_record!(Cols3;  A:0, B:1, C:2);
impl_csv_record!(Cols4;  A:0, B:1, C:2, D:3);
impl_csv_record!(Cols5;  A:0, B:1, C:2, D:3, E:4);
impl_csv_record!(Cols6;  A:0, B:1, C:2, D:3, E:4, F:5);
impl_csv_record!(Cols7;  A:0, B:1, C:2, D:3, E:4, F:5, G:6);
impl_csv_record!(Cols8;  A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7);
impl_csv_record!(Cols9;  A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8);
impl_csv_record!(Cols10; A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9);
impl_csv_record!(Cols11; A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10);
impl_csv_record!(Cols12; A:0, B:1, C:2, D:3, E:4, F:5, G:6, H:7, I:8, J:9, K:10, L:11);

// ── CsvWriter（元组结构，持有输出路径）───────────────────────────────────────
/// CSV 临时文件入口，由 `ExportTaskModel` 自动生成文件名。
///
/// 文件名格式：`$TEMP/export_{export_type}_{id}.csv`
pub struct CsvWriter(pub PathBuf);

impl CsvWriter {
    /// 根据 `record.export_type` + `record.id` 生成临时路径
    pub fn new(record: &ExportTaskModel) -> Self {
        let path =
            std::env::temp_dir().join(format!("export_{}_{}.csv", record.export_type, record.id));
        Self(path)
    }

    /// 创建文件，写 UTF-8 BOM + 表头行，返回 `CsvRowWriter<H::Arity>`。
    ///
    /// `header` 的元组类型决定后续所有行写入的列数约束（编译期检查）。
    pub async fn header<H: CsvRecord>(
        self,
        header: H,
    ) -> Result<CsvRowWriter<H::Arity>, FileManagerError> {
        let mut file = tokio::fs::File::create(&self.0).await?;
        file.write_all(b"\xEF\xBB\xBF").await?; // UTF-8 BOM
        file.write_all(to_line(&header.to_fields()).as_bytes())
            .await?;
        Ok(CsvRowWriter {
            file,
            path: self.0,
            _arity: PhantomData,
        })
    }
}

// ── CsvRowWriter（行写入器）──────────────────────────────────────────────────
/// 行写入器，`A` 为列数 Marker，由 `CsvWriter::header` 自动推断，无需手写。
pub struct CsvRowWriter<A> {
    file: tokio::fs::File,
    path: PathBuf,
    _arity: PhantomData<A>,
}

impl<A> CsvRowWriter<A> {
    /// 写入单行。
    ///
    /// 行的元组类型 `R` 必须与 header 等宽（`R::Arity == A`），否则编译报错。
    pub async fn write_row<R: CsvRecord<Arity = A>>(
        &mut self,
        row: R,
    ) -> Result<(), FileManagerError> {
        self.file
            .write_all(to_line(&row.to_fields()).as_bytes())
            .await?;
        Ok(())
    }

    /// 批量写入。接受任何可迭代的行集合（`Vec`、迭代器等），同上列数约束。
    pub async fn write_batch<R: CsvRecord<Arity = A>>(
        &mut self,
        rows: impl IntoIterator<Item = R>,
    ) -> Result<(), FileManagerError> {
        let mut buf = String::new();
        for row in rows {
            buf.push_str(&to_line(&row.to_fields()));
            if buf.len() > 1024 * 256 {
                self.file.write_all(buf.as_bytes()).await?;
                buf.clear();
            }
        }
        if !buf.is_empty() {
            self.file.write_all(buf.as_bytes()).await?;
        }
        Ok(())
    }

    /// flush 并返回文件路径（交由调用方存入 lsys-file）。
    pub async fn finish(mut self) -> Result<PathBuf, FileManagerError> {
        self.file.flush().await?;
        Ok(self.path)
    }
}

// ── 辅助函数 ─────────────────────────────────────────────────────────────────

/// 将已转义的字段列表拼成一行（逗号分隔 + 换行符）
fn to_line(fields: &[String]) -> String {
    let mut line = fields.join(",");
    line.push('\n');
    line
}

/// CSV 字段转义：含逗号、引号或换行时用双引号包裹，内部引号加倍
fn csv_escape(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_owned()
    }
}
