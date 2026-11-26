use crate::i18n::Language;
use crate::storage::Vault;
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use anyhow::{Context, Result, bail};
use argon2::Argon2;
use directories::ProjectDirs;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

const SALT_LEN: usize = 16;
const NONCE_LEN: usize = 12;
const MAGIC: &[u8; 4] = b"RMV3";
const LEGACY_MAGIC: &[u8; 4] = b"RMV2";

pub fn default_vault_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "cleboost", "rustmius")
        .context("Failed to determine application data directory")?;
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("vault.json"))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppSettings {
    pub language: Language,
    pub text_size: f32,
    pub copy_on_select: bool,
    pub bright_bold: bool,
    pub auto_unlock_password: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: Language::En,
            text_size: 14.0,
            copy_on_select: false,
            bright_bold: false,
            auto_unlock_password: None,
        }
    }
}

pub fn default_settings_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "cleboost", "rustmius")
        .context("Failed to determine application data directory")?;
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("settings.json"))
}

pub fn write_secure(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn save_app_settings(path: &Path, settings: &AppSettings) -> Result<()> {
    let json = serde_json::to_string_pretty(settings)?;
    write_secure(path, json.as_bytes())
}

pub fn load_app_settings(path: &Path) -> AppSettings {
    fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn default_sync_config_path() -> Result<PathBuf> {
    let dirs = ProjectDirs::from("com", "cleboost", "rustmius")
        .context("Failed to determine application data directory")?;
    let data_dir = dirs.data_dir();
    fs::create_dir_all(data_dir)?;
    Ok(data_dir.join("sync.enc"))
}

pub fn save_plain_vault(path: &Path, vault: &Vault) -> Result<()> {
    let json = serde_json::to_string_pretty(vault)?;
    write_secure(path, json.as_bytes())
}

pub fn load_plain_vault(path: &Path) -> Result<Vault> {
    if !path.exists() {
        return Ok(Vault::new());
    }
    let data = fs::read_to_string(path)?;
    let vault: Vault = serde_json::from_str(&data)?;
    Ok(vault)
}

fn derive_key(password: &str, salt: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    let mut key = Zeroizing::new([0u8; 32]);
    Argon2::default()
        .hash_password_into(password.as_bytes(), salt, key.as_mut())
        .map_err(|e| anyhow::anyhow!("Argon2 key derivation failed: {e}"))?;
    Ok(key)
}

pub fn encrypt_bytes(plaintext: &[u8], password: &str) -> Result<Vec<u8>> {
    let mut salt = [0u8; SALT_LEN];
    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut nonce_bytes);

    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_ref());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| anyhow::anyhow!("Encryption failed: {e}"))?;

    let mut out = Vec::with_capacity(MAGIC.len() + SALT_LEN + NONCE_LEN + ciphertext.len());
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&salt);
    out.extend_from_slice(&nonce_bytes);
    out.extend_from_slice(&ciphertext);
    Ok(out)
}

pub fn decrypt_bytes(data: &[u8], password: &str) -> Result<Zeroizing<Vec<u8>>> {
    let header_len = MAGIC.len() + SALT_LEN + NONCE_LEN;
    if data.len() < header_len
        || (&data[..MAGIC.len()] != MAGIC && &data[..MAGIC.len()] != LEGACY_MAGIC)
    {
        bail!("Invalid encrypted file format");
    }

    let salt = &data[MAGIC.len()..MAGIC.len() + SALT_LEN];
    let nonce_bytes = &data[MAGIC.len() + SALT_LEN..header_len];
    let ciphertext = &data[header_len..];

    let key_bytes = derive_key(password, salt)?;
    let key = Key::<Aes256Gcm>::from_slice(key_bytes.as_ref());
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| anyhow::anyhow!("Incorrect master password or corrupted file"))?;
    Ok(Zeroizing::new(plaintext))
}

pub fn encrypt_vault(vault: &Vault, password: &str) -> Result<Vec<u8>> {
    let json = Zeroizing::new(serde_json::to_vec(vault)?);
    encrypt_bytes(&json, password)
}

pub fn decrypt_vault(data: &[u8], password: &str) -> Result<Vault> {
    let plaintext = decrypt_bytes(data, password)?;
    let vault: Vault = serde_json::from_slice(&plaintext)?;
    Ok(vault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Host;

    #[test]
    fn roundtrip_encrypt_decrypt() {
        let mut vault = Vault::new();
        vault
            .hosts
            .push(Host::new("prod-web", "10.0.0.1", "ubuntu"));

        let encrypted = encrypt_vault(&vault, "super-secret-password").unwrap();
        let decrypted = decrypt_vault(&encrypted, "super-secret-password").unwrap();

        assert_eq!(decrypted.hosts.len(), 1);
        assert_eq!(decrypted.hosts[0].label, "prod-web");
        assert_eq!(decrypted.hosts[0].hostname, "10.0.0.1");
    }

    #[test]
    fn wrong_password_fails() {
        let vault = Vault::new();
        let encrypted = encrypt_vault(&vault, "correct-password").unwrap();
        assert!(decrypt_vault(&encrypted, "wrong-password").is_err());
    }
}
