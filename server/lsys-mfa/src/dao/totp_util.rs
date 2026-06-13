use base32::Alphabet;
use hmac::{Hmac, KeyInit, Mac};
use sha1::Sha1;

use super::{MfaError, MfaResult};

type HmacSha1 = Hmac<Sha1>;

pub fn decode_base32(secret: &str) -> MfaResult<Vec<u8>> {
    let s = secret.trim().replace([' ', '-'], "").to_uppercase();
    base32::decode(Alphabet::Rfc4648 { padding: false }, s.as_str()).ok_or(MfaError::SecretInvalid)
}

pub fn totp_code(secret: &[u8], step: u64, digits: u32) -> MfaResult<String> {
    let mut counter = [0u8; 8];
    counter.copy_from_slice(&step.to_be_bytes());

    let mut mac = HmacSha1::new_from_slice(secret).map_err(|_| MfaError::SecretInvalid)?;
    mac.update(&counter);
    let hash = mac.finalize().into_bytes();

    let offset = (hash[hash.len() - 1] & 0x0f) as usize;
    let bin_code = ((hash[offset] as u32 & 0x7f) << 24)
        | ((hash[offset + 1] as u32) << 16)
        | ((hash[offset + 2] as u32) << 8)
        | (hash[offset + 3] as u32);

    let modulo = 10u32.pow(digits);
    let otp = bin_code % modulo;
    Ok(format!("{:01$}", otp, digits as usize))
}
