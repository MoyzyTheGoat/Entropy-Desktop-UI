use sha2::{Sha256, Digest};
use aes_gcm::{Aes256Gcm, Nonce, aead::{Aead, KeyInit}};
use rand::{RngCore, thread_rng};
use crate::protocol::types::MediaKeyBundle;
use crate::protocol::utils::{encode_b64, decode_b64};
use rusqlite::Connection;

pub fn encrypt_media(
    _conn: &Connection,
    data: &[u8],
    file_name: &str,
    file_type: &str
) -> Result<(Vec<u8>, MediaKeyBundle), String> {
    let mut rng = thread_rng();
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);
    
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    rng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    let ciphertext = cipher.encrypt(nonce, data).map_err(|e| e.to_string())?;
    
    let mut hasher = Sha256::new();
    hasher.update(data);
    let digest = hasher.finalize();

    let bundle = MediaKeyBundle {
        key: encode_b64(&key_bytes),
        nonce: encode_b64(&nonce_bytes),
        digest: encode_b64(&digest),
        file_name: file_name.to_string(),
        file_type: file_type.to_string(),
        is_chunked: false,
        chunk_size: None
    };

    Ok((ciphertext, bundle))
}

pub fn decrypt_media(
    _conn: &Connection,
    ciphertext: &[u8],
    bundle: &MediaKeyBundle
) -> Result<Vec<u8>, String> {
    if bundle.is_chunked {
        return Err("Use decrypt_media_chunk for chunked media".to_string());
    }

    let key_bytes = decode_b64(&bundle.key)?;
    let nonce_bytes = decode_b64(&bundle.nonce)?;
    
    let cipher = Aes256Gcm::new_from_slice(&key_bytes).map_err(|e| e.to_string())?;
    let nonce = Nonce::from_slice(&nonce_bytes);
    let pt = cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())?;
    
    let mut hasher = Sha256::new();
    hasher.update(&pt);
    let digest = hasher.finalize();

    if encode_b64(&digest) != bundle.digest {
        return Err("Media digest mismatch".to_string());
    }

    Ok(pt)
}

pub fn encrypt_media_chunk(
    key: &[u8],
    base_nonce: &[u8],
    chunk_index: u32,
    data: &[u8]
) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    // Use first 8 bytes of base_nonce and last 4 bytes for chunk_index
    nonce_bytes[..8].copy_from_slice(&base_nonce[..8]);
    nonce_bytes[8..12].copy_from_slice(&chunk_index.to_be_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    cipher.encrypt(nonce, data).map_err(|e| e.to_string())
}

pub fn decrypt_media_chunk(
    key: &[u8],
    base_nonce: &[u8],
    chunk_index: u32,
    ciphertext: &[u8]
) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| e.to_string())?;
    let mut nonce_bytes = [0u8; 12];
    nonce_bytes[..8].copy_from_slice(&base_nonce[..8]);
    nonce_bytes[8..12].copy_from_slice(&chunk_index.to_be_bytes());
    let nonce = Nonce::from_slice(&nonce_bytes);
    
    cipher.decrypt(nonce, ciphertext).map_err(|e| e.to_string())
}
