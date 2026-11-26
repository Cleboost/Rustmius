use crate::i18n::Language;
use crate::storage::Vault;
use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};
use anyhow::{Context, Result, bail};
use argon2::Argon2;
use directories::ProjectDirs;
use rand::Rng;
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
    use std::io::Write;

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("vault");
    let tmp_path = path.with_file_name(format!(".{file_name}.tmp.{}", std::process::id()));

    let write_result = (|| -> Result<()> {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600));
        }
        fs::rename(&tmp_path, path)?;
        Ok(())
    })();

    if write_result.is_err() {
        let _ = fs::remove_file(&tmp_path);
    }
    write_result
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

fn backup_path(path: &Path) -> PathBuf {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("vault");
    path.with_file_name(format!("{file_name}.bak"))
}

fn quarantine_corrupted_file(path: &Path) {
    let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("vault");
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let corrupted = path.with_file_name(format!("{file_name}.corrupted.{ts}"));
    let _ = fs::rename(path, corrupted);
}

/// Best-effort recovery of complete JSON objects from a truncated array under `"<key>": [ ... ]`.
fn recover_json_array<T: serde::de::DeserializeOwned>(data: &str, key: &str) -> Vec<T> {
    let pattern = format!("\"{key}\"");
    let Some(key_pos) = data.find(&pattern) else {
        return Vec::new();
    };
    let after_key = &data[key_pos + pattern.len()..];
    let Some(bracket_rel) = after_key.find('[') else {
        return Vec::new();
    };
    let array_slice = &after_key[bracket_rel + 1..];

    let mut items = Vec::new();
    let mut in_string = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut obj_start: Option<usize> = None;

    for (idx, ch) in array_slice.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '{' => {
                if depth == 0 {
                    obj_start = Some(idx);
                }
                depth += 1;
            }
            '}' => {
                if depth > 0 {
                    depth -= 1;
                    if depth == 0 {
                        if let Some(start) = obj_start.take() {
                            if let Ok(item) = serde_json::from_str::<T>(&array_slice[start..=idx]) {
                                items.push(item);
                            }
                        }
                    }
                }
            }
            ']' if depth == 0 => break,
            _ => {}
        }
    }

    items
}

fn recover_truncated_plain_vault(data: &str) -> Option<Vault> {
    let hosts = recover_json_array::<crate::storage::Host>(data, "hosts");
    let groups = recover_json_array::<crate::storage::Group>(data, "groups");
    if hosts.is_empty() && groups.is_empty() {
        return None;
    }
    Some(Vault {
        version: 1,
        hosts,
        groups,
        deleted_hosts: std::collections::HashMap::new(),
        known_hosts: Vec::new(),
    })
}

pub fn save_plain_vault(path: &Path, vault: &Vault) -> Result<()> {
    if path.exists() {
        if let Ok(existing) = fs::read(path) {
            let _ = write_secure(&backup_path(path), &existing);
        }
    }
    let json = serde_json::to_string_pretty(vault)?;
    write_secure(path, json.as_bytes())
}

pub fn load_plain_vault(path: &Path) -> Result<Vault> {
    let bak = backup_path(path);
    if !path.exists() {
        if bak.exists() {
            if let Ok(bak_data) = fs::read_to_string(&bak) {
                if let Ok(vault) = serde_json::from_str::<Vault>(&bak_data) {
                    return Ok(vault);
                }
            }
        }
        return Ok(Vault::new());
    }

    let data = fs::read_to_string(path)?;
    match serde_json::from_str::<Vault>(&data) {
        Ok(vault) => Ok(vault),
        Err(primary_err) => {
            if bak.exists() {
                if let Ok(bak_data) = fs::read_to_string(&bak) {
                    if let Ok(vault) = serde_json::from_str::<Vault>(&bak_data) {
                        quarantine_corrupted_file(path);
                        return Ok(vault);
                    }
                }
            }
            if let Some(recovered) = recover_truncated_plain_vault(&data) {
                quarantine_corrupted_file(path);
                return Ok(recovered);
            }
            Err(primary_err.into())
        }
    }
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
    rand::rng().fill_bytes(&mut salt);
    rand::rng().fill_bytes(&mut nonce_bytes);

    let key_bytes = derive_key(password, &salt)?;
    let key = Key::<Aes256Gcm>::try_from(key_bytes.as_ref())?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::try_from(nonce_bytes.as_slice())?;

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
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
    let key = Key::<Aes256Gcm>::try_from(key_bytes.as_ref())?;
    let cipher = Aes256Gcm::new(&key);
    let nonce = Nonce::try_from(nonce_bytes)?;

    let plaintext = cipher
        .decrypt(&nonce, ciphertext)
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

    #[test]
    fn load_plain_vault_falls_back_to_backup_when_corrupted() {
        let dir = std::env::temp_dir().join(format!("rustmius_test_bak_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let vault_path = dir.join("vault.json");

        let mut v1 = Vault::new();
        v1.hosts.push(Host::new("host-1", "10.0.0.1", "root"));
        save_plain_vault(&vault_path, &v1).unwrap();

        let mut v2 = v1.clone();
        v2.hosts.push(Host::new("host-2", "10.0.0.2", "ubuntu"));
        save_plain_vault(&vault_path, &v2).unwrap();

        // Corrupt the primary vault file
        fs::write(&vault_path, "{ truncated garbage").unwrap();

        let loaded = load_plain_vault(&vault_path).unwrap();
        assert_eq!(loaded.hosts.len(), 1);
        assert_eq!(loaded.hosts[0].label, "host-1");

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_plain_vault_recovers_hosts_from_truncated_json() {
        let dir = std::env::temp_dir().join(format!("rustmius_test_trunc_{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let vault_path = dir.join("vault.json");

        let mut vault = Vault::new();
        vault.hosts.push(Host::new("alpha", "10.0.0.1", "root"));
        vault.hosts.push(Host::new("beta", "10.0.0.2", "admin"));
        let json = serde_json::to_string_pretty(&vault).unwrap();

        // Truncate inside the second host
        let cut = json.rfind("\"beta\"").unwrap() + 10;
        fs::write(&vault_path, &json[..cut]).unwrap();

        let recovered = load_plain_vault(&vault_path).unwrap();
        assert_eq!(recovered.hosts.len(), 1);
        assert_eq!(recovered.hosts[0].label, "alpha");

        let _ = fs::remove_dir_all(&dir);
    }
}
