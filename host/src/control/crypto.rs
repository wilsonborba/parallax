use hmac::{Hmac, Mac};
use hkdf::Hkdf;
use pbkdf2::pbkdf2_hmac;
use rand::rngs::OsRng;
use rand::RngCore;
use sha2::Sha256;

pub const NONCE_LEN: usize = 32;
pub const KEY_LEN: usize = 32;
pub const HMAC_LEN: usize = 32;
pub const PBKDF2_ITERATIONS: u32 = 100_000;
pub const PBKDF2_SALT: &[u8] = b"parallax-control";
pub const HKDF_INFO: &[u8] = b"parallax-control-auth";

type HmacSha256 = Hmac<Sha256>;

pub fn generate_nonce() -> [u8; NONCE_LEN] {
    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);
    nonce
}

pub fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32, key_len: usize) -> Vec<u8> {
    let mut out = vec![0u8; key_len];
    pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut out);
    out
}

pub fn hkdf_sha256(ikm: &[u8], salt: &[u8], info: &[u8], key_len: usize) -> Vec<u8> {
    let hkdf = Hkdf::<Sha256>::new(Some(salt), ikm);
    let mut out = vec![0u8; key_len];
    hkdf.expand(info, &mut out)
        .expect("HKDF expand should only fail on invalid output length");
    out
}

pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take any key size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

pub fn verify_hmac_sha256(key: &[u8], data: &[u8], expected: &[u8]) -> bool {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take any key size");
    mac.update(data);
    mac.verify_slice(expected).is_ok()
}

pub fn derive_master_key(pairing_token: &str) -> Vec<u8> {
    pbkdf2_sha256(
        pairing_token.as_bytes(),
        PBKDF2_SALT,
        PBKDF2_ITERATIONS,
        KEY_LEN,
    )
}

pub fn derive_session_key(master_key: &[u8], nonce: &[u8]) -> Vec<u8> {
    hkdf_sha256(master_key, nonce, HKDF_INFO, KEY_LEN)
}
