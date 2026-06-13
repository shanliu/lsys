/// 生成 CSV 导出表头字符串或元组，将字段名映射为多语言翻译字符串。
///
/// 翻译 key 约定：`export-{任务名}-{字段名}`，crate 名取调用处的 `CARGO_PKG_NAME`。
/// 加入任务名可避免多个导出任务中同名字段的翻译 key 冲突。
///
/// # 两种形式
///
/// **单字段** → 返回 `String`：
/// ```ignore
/// let header = export_header!(fluent, EXPORT_TYPE_USER_FILE_LIST, "id");
/// ```
///
/// **多字段** → 返回元组，内部复用单字段形式：
/// ```ignore
/// let mut w = CsvWriter::new(&record)
///     .header(export_header!(fluent, EXPORT_TYPE_USER_FILE_LIST, "id", "file_name", "add_time"))
///     .await?;
/// ```
#[macro_export]
macro_rules! export_header {
    // 单字段形式：返回 String
    ($fluent:expr, $task:expr, $field:literal $(,)?) => {
        $fluent.format_message(&lsys_core::fluents::FluentMessage {
            id: format!("export-{}-{}", $task, $field),
            crate_name: env!("CARGO_PKG_NAME").to_string(),
            data: vec![],
        })
    };
    // 多字段形式：返回元组，复用单字段形式
    ($fluent:expr, $task:expr, $($field:literal),+ $(,)?) => {
        ($(export_header!($fluent, $task, $field),)+)
    };
}
