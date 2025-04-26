use rand::seq::IndexedRandom;

pub enum RandType {
    Number,
    Upper,
    Lower,
    UpperNumber,
    LowerNumber,
    UpperHex,
    LowerHex,
}

pub fn rand_str(rand_type: RandType, len: usize) -> String {
    let base_str = match rand_type {
        RandType::Number => "0123456789",
        RandType::Upper => "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
        RandType::Lower => "abcdefghijklmnopqrstuvwxyz",
        RandType::UpperNumber => "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789",
        RandType::LowerNumber => "abcdefghijklmnopqrstuvwxyz0123456789",
        RandType::UpperHex => "ABCDEF0123456789",
        RandType::LowerHex => "abcdef0123456789",
    };
    let mut rng = &mut rand::rng();
    String::from_utf8(
        base_str
            .as_bytes()
            .choose_multiple(&mut rng, len)
            .cloned()
            .collect(),
    )
    .unwrap_or_default()
}

pub const CLEAR_TS_SPACE: u32 = 1 << 0; // Trim & Squash Space（去除空格制表符、合并空格）
pub const CLEAR_NL: u32 = 1 << 1; // Remove NewLines（去除换行符）
pub const CLEAR_XSS: u32 = 1 << 2; // Sanitize XSS（防止XSS注入）
pub const CLEAR_SP_CHAR: u32 = 1 << 3; // Remove Special Chars（删除特殊符号）
pub const CLEAR_BASE: u32 = CLEAR_TS_SPACE | CLEAR_NL | CLEAR_XSS | CLEAR_SP_CHAR; //基本输入过滤
pub const CLEAR_IDENT: u32 = 1 << 4 | CLEAR_TS_SPACE | CLEAR_NL; //标识符字符过滤
pub fn clear_string(input: impl ToString, flags: u32) -> String {
    let mut s = input.to_string();
    // CLEAR_XSS: 替换敏感字符为安全字符
    if (flags & CLEAR_XSS) != 0 {
        s = s
            .replace('>', "]")
            .replace('<', "[")
            .replace(['&', '"', '\''], " ");
    }
    // CLEAR_NL: 删除换行符和\r
    if (flags & CLEAR_NL) != 0 {
        s = s.replace(['\n', '\r'], " ");
    }
    // CLEAR_SP_CHAR: 删除特殊符号
    if (flags & CLEAR_SP_CHAR) != 0 {
        let sp_chars = ['&', '^', '`', '/', '\\', '\"', '\''];
        s = s.chars().filter(|c| !sp_chars.contains(c)).collect();
    }
    // CLEAR_IDENT: 转为允许数字开头的类标识符格式
    if (flags & CLEAR_IDENT) != 0 {
        s = s
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '_')
            .collect()
    }
    // CLEAR_TS_SPACE: 去前后空格/制表符，合并中间空格
    if (flags & CLEAR_TS_SPACE) != 0 {
        s = s.trim().to_string();
        s = s.split_whitespace().collect::<Vec<_>>().join(" ");
    }
    s
}
