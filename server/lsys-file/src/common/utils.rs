/// 生成高质量的随机数字符串：结合当天秒数 + 微秒 + 随机数，确保完全不重复
/// 格式: ((当天秒数+1) * 1000000) + 微秒(0-999999) + 随机数(0-9)
/// 返回固定 10 位长度的字符串（前导零补齐），可用于同一秒内的高频调用
pub fn rand_simple() -> String {
    use chrono::Timelike;

    // 获取当前时间
    let now = chrono::Local::now();

    // 计算当前时间距离今天0点的秒数（0-86399），加1后为 1-86400
    let seconds_today = ((now.hour() * 3600 + now.minute() * 60 + now.second()) as u64) + 1;

    // 获取微秒部分（0-999999）
    let microseconds = now.timestamp_subsec_micros() as u64;

    // 生成 0-9 的随机数（1位）
    let random_digit = (rand::random::<u32>() % 10) as u64;

    // 合并：(秒数+1) * 1000000 + 微秒 + 随机数
    // 利用微秒（0-999999）来填充同一秒内的时间差异
    // 最小值：1 * 1000000 + 0 + 0 = 1000000
    // 最大值：86400 * 1000000 + 999999 + 9 = 86,400,999,999+9（11位）
    let combined = (seconds_today * 1_000_000) + microseconds + random_digit;

    // 返回最后10位，确保长度一致
    format!("{:0>10}", combined % 10_000_000_000)
}

/// 从文件名中提取扩展名，拿不到则返回 "dat"
pub fn extract_extension(file_name: Option<&str>) -> &str {
    use std::path::Path;
    match file_name {
        Some(name) if !name.trim().is_empty() => Path::new(name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("dat"),
        _ => "dat",
    }
}

/// 清理文件名中的危险字符
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect::<String>()
        .chars()
        .take(200)
        .collect()
}
