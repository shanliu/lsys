use std::path::PathBuf;

use crate::{AreaError, AreaResult};

#[allow(dead_code)]
pub(crate) fn read_file_md5(path: &PathBuf) -> String {
    use sha2::{Digest, Sha256};
    use std::{fs, io};
    if let Ok(mut file) = fs::File::open(path) {
        let mut hasher = Sha256::new();
        if io::copy(&mut file, &mut hasher).is_ok() {
            return format!("{:x}", hasher.finalize());
        }
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
