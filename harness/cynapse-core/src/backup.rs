//! Authenticated Encrypted Backup & Restore Module for Cynapse
//!
//! Provides authenticated AES-256-GCM encryption with PBKDF2-HMAC-SHA256
//! key derivation and cryptographically secure random salts and nonces.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use anyhow::{bail, Context, Result};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use sha2::Sha256;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub const MAGIC_V2: &[u8] = b"CYNAPSE_ENC_v2";
pub const MAGIC_V1: &[u8] = b"CYNAPSE_ENC_v1";
const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const PBKDF2_ROUNDS: u32 = 100_000;

pub struct EncryptedBackup;

impl EncryptedBackup {
    /// Derive a 32-byte AES-256 key from a passphrase and salt using PBKDF2-HMAC-SHA256.
    pub fn derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32] {
        let mut key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, PBKDF2_ROUNDS, &mut key);
        key
    }

    /// Encrypt a source file (e.g. dendrite.db) with AES-256-GCM and write to output_path.
    pub fn create_backup(source_path: &Path, output_path: &Path, passkey: &str) -> Result<u64> {
        let trimmed_key = passkey.trim();
        if trimmed_key.is_empty() {
            bail!("Passphrase cannot be empty");
        }

        let mut input_file = File::open(source_path)
            .with_context(|| format!("Failed to open source file for backup: {:?}", source_path))?;
        let mut plaintext = Vec::new();
        input_file.read_to_end(&mut plaintext)?;

        let mut salt = [0u8; SALT_LEN];
        let mut nonce_bytes = [0u8; NONCE_LEN];
        let mut rng = rand::thread_rng();
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut nonce_bytes);

        let key = Self::derive_key(trimmed_key, &salt);
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| anyhow::anyhow!("Failed to initialize AES-256-GCM: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("Encryption failed: {}", e))?;

        if let Some(parent) = output_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let mut output_file = File::create(output_path)
            .with_context(|| format!("Failed to create backup file: {:?}", output_path))?;

        output_file.write_all(MAGIC_V2)?;
        output_file.write_all(&salt)?;
        output_file.write_all(&nonce_bytes)?;
        output_file.write_all(&ciphertext)?;
        output_file.flush()?;

        let total_size = MAGIC_V2.len() + SALT_LEN + NONCE_LEN + ciphertext.len();
        Ok(total_size as u64)
    }

    /// Decrypt an encrypted backup file and write the restored plaintext to output_path.
    pub fn restore_backup(encrypted_path: &Path, output_path: &Path, passkey: &str) -> Result<()> {
        let trimmed_key = passkey.trim();
        if trimmed_key.is_empty() {
            bail!("Passphrase cannot be empty");
        }

        let mut input_file = File::open(encrypted_path)
            .with_context(|| format!("Failed to open backup file: {:?}", encrypted_path))?;
        let mut buffer = Vec::new();
        input_file.read_to_end(&mut buffer)?;

        if buffer.starts_with(MAGIC_V2) {
            let header_len = MAGIC_V2.len();
            if buffer.len() < header_len + SALT_LEN + NONCE_LEN {
                bail!("Corrupted backup file: header too short");
            }

            let salt = &buffer[header_len..header_len + SALT_LEN];
            let nonce_bytes = &buffer[header_len + SALT_LEN..header_len + SALT_LEN + NONCE_LEN];
            let ciphertext = &buffer[header_len + SALT_LEN + NONCE_LEN..];

            let key = Self::derive_key(trimmed_key, salt);
            let cipher = Aes256Gcm::new_from_slice(&key)
                .map_err(|e| anyhow::anyhow!("Failed to initialize AES-256-GCM: {}", e))?;
            let nonce = Nonce::from_slice(nonce_bytes);

            let plaintext = cipher
                .decrypt(nonce, ciphertext)
                .map_err(|_| anyhow::anyhow!("Decryption failed: invalid passphrase or corrupted file"))?;

            if let Some(parent) = output_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }

            let mut output_file = File::create(output_path)
                .with_context(|| format!("Failed to create restored file: {:?}", output_path))?;
            output_file.write_all(&plaintext)?;
            output_file.flush()?;

            Ok(())
        } else if buffer.starts_with(MAGIC_V1) {
            // Legacy V1 (XOR) decryption fallback
            use sha2::Digest;
            let encrypted_payload = &buffer[MAGIC_V1.len()..];
            let mut hasher = Sha256::new();
            hasher.update(trimmed_key.as_bytes());
            hasher.update(b"CYNAPSE_SECURE_SALT_2026");
            let key = hasher.finalize();

            let mut decrypted_payload = Vec::with_capacity(encrypted_payload.len());
            for (i, byte) in encrypted_payload.iter().enumerate() {
                let key_byte = key[i % key.len()];
                decrypted_payload.push(byte ^ key_byte);
            }

            if let Some(parent) = output_path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let mut output_file = File::create(output_path)?;
            output_file.write_all(&decrypted_payload)?;
            output_file.flush()?;
            Ok(())
        } else {
            bail!("Unrecognized backup format or missing magic header");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_backup_roundtrip_aes256() {
        let dir = tempdir().unwrap();
        let src_path = dir.path().join("test_dendrite.db");
        let enc_path = dir.path().join("backup.enc");
        let restored_path = dir.path().join("restored.db");

        let test_data = b"Cynapse Dendrite Graph Memory Test Data 1234567890 \x00\xff\xfe";
        std::fs::write(&src_path, test_data).unwrap();

        let passkey = "SuperSecretPassphrase2026!";
        let bytes_written = EncryptedBackup::create_backup(&src_path, &enc_path, passkey).unwrap();
        assert!(bytes_written > test_data.len() as u64);

        // Verify wrong passkey fails
        assert!(EncryptedBackup::restore_backup(&enc_path, &restored_path, "WrongPassword").is_err());

        // Verify correct passkey succeeds
        EncryptedBackup::restore_backup(&enc_path, &restored_path, passkey).unwrap();
        let recovered = std::fs::read(&restored_path).unwrap();
        assert_eq!(recovered, test_data);
    }
}