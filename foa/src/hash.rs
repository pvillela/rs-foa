use base64ct::{Base64, Encoding};
use sha2::{Digest, Sha256};

/// Computes the MD5 hash of a string array.
/// When computing the hash, elements of the array are interspersed with 0 bytes to differentiate,
/// for example, between ["hello", "world"] and ["helloworld"].
pub fn hash_md5_of_str_arr<T>(arr: &[T]) -> [u8; 16]
where
    T: AsRef<str>,
{
    let mut md5 = md5::Context::new();
    for (i, data) in arr.iter().enumerate() {
        if i > 0 {
            md5.consume([0_u8; 1]);
        }
        md5.consume(data.as_ref());
    }
    md5.compute().into()
}

/// Computes the SHA256 hash of a string array.
/// When computing the hash, elements of the array are interspersed with 0 bytes to differentiate,
/// for example, between ["hello", "world"] and ["helloworld"].
pub fn hash_sha256_of_str_arr<T>(arr: &[T]) -> [u8; 32]
where
    T: AsRef<str>,
{
    let mut hasher = Sha256::new();
    for (i, data) in arr.iter().enumerate() {
        if i > 0 {
            hasher.update([0_u8; 1]);
        }
        hasher.update(data.as_ref());
    }
    hasher.finalize().into()
}

/// Encodes a byte array as a lower hex string.
pub fn hex_lower_str_of_u8_arr(arr: &[u8]) -> String {
    let mut hex_str = String::with_capacity(32);
    for b in arr {
        hex_str.push_str(&format!("{:x}", b));
    }
    hex_str
}

/// Encodes a byte array as a Base64 string, truncating it to a given max size.
/// If the max size is greater than or equal to the length of the array then
/// the encoding is returned without truncation.
pub fn base64_encode_trunc_of_u8_arr(arr: &[u8], max_size: usize) -> String {
    let trunc = arr.len().min(max_size);
    Base64::encode_string(&arr[0..trunc])
}
