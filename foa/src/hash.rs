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
