pub const STRING_CLEAR_SPACE: u32 = 1 << 0; // Trim & Squash Space（去除空格制表符、合并空格）
pub const STRING_CLEAR_NL: u32 = 1 << 1; // Remove NewLines（去除换行符）
pub const STRING_CLEAR_FORMAT: u32 = 1 << 2 | STRING_CLEAR_SPACE | STRING_CLEAR_NL; // 去除换行,连续空格,制表符等格式符号
pub const STRING_CLEAR_XSS: u32 = 1 << 3; // Sanitize XSS（防止XSS注入）
pub enum StringClear {
    Option(u32), //可选过滤组合 STRING_CLEAR_* 常量
    LikeKeyWord, //MYSQL LIKE 关键字过滤
    Ident,       //标识符 ,跟ValidPatternRule::Ident 保持一致
}

pub fn string_clear(input: impl ToString, clear_flag: StringClear, take: Option<usize>) -> String {
    let mut s = input.to_string();
    match clear_flag {
        StringClear::Option(flags) => {
            if (flags & STRING_CLEAR_XSS) != 0 {
                s = s.replace('>', "]").replace('<', "[").replace('&', " ");
            }
            // CLEAR_NL: 删除换行符和\r
            if (flags & STRING_CLEAR_FORMAT) != 0 {
                s = s.replace(['\t', '\0', '\\'], " ");
            }
            // CLEAR_NL: 删除换行符和\r
            if (flags & STRING_CLEAR_NL) != 0 {
                s = s.replace(['\n', '\r'], " ");
            }
            // CLEAR_TS_SPACE: 去前后空格/制表符，合并中间空格
            if (flags & STRING_CLEAR_SPACE) != 0 {
                s = s.split_whitespace().collect::<Vec<_>>().join(" ");
                s = s.trim().to_string();
            }
        }
        StringClear::LikeKeyWord => {
            s = s.replace('\\', " ").replace('%', "\\%").replace('_', "\\_");
            s = string_clear(s, StringClear::Option(STRING_CLEAR_FORMAT), take);
        }
        StringClear::Ident => {
            s = s
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '.')
                .collect()
        }
    }
    if let Some(len) = take {
        s = s.chars().take(len).collect::<String>();
    }
    s
}
