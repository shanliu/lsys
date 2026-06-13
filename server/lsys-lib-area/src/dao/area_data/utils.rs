use std::path::PathBuf;

use crate::{AreaError, AreaResult};

#[allow(dead_code)]
pub(crate) fn read_file_md5(path: &PathBuf) -> String {
    use sha2::{Digest, Sha256};
    use std::{fmt::Write, fs, io::Read};
    if let Ok(mut file) = fs::File::open(path) {
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];
        loop {
            match file.read(&mut buffer) {
                Ok(0) => break,
                Ok(n) => hasher.update(&buffer[..n]),
                Err(_) => return "".to_string(),
            }
        }
        let hash = hasher.finalize();
        let mut hex_str = String::with_capacity(hash.len() * 2);
        for b in hash {
            let _ = write!(hex_str, "{:02x}", b);
        }
        return hex_str;
    }
    "".to_string()
}
#[allow(dead_code)]
pub(crate) fn read_file(path: &PathBuf) -> AreaResult<Vec<u8>> {
    std::fs::read(path).map_err(|e| AreaError::System(e.to_string()))
}

pub(crate) fn de_gz_data(zip_data: Vec<u8>) -> AreaResult<Vec<u8>> {
    let mut s = vec![];
    use std::io::Read;
    let mut gz = flate2::read::GzDecoder::new(&zip_data[..]);
    gz.read_to_end(&mut s)
        .map_err(|e| AreaError::System(e.to_string()))?;
    Ok(s)
}
pub(crate) fn en_name_keyword(input: &str) -> String {
    let mut result = String::new();
    let mut prev_char = ' ';

    for c in input.chars() {
        if c.is_uppercase() {
            if !prev_char.is_whitespace() {
                result.push(' ');
            }
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
        prev_char = c;
    }

    result
}
