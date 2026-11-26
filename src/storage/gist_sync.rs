//! GitHub Gist E2E encrypted vault synchronization.

use crate::storage::store::{
    decrypt_bytes, decrypt_vault, encrypt_bytes, encrypt_vault, write_secure,
};
use crate::storage::{Group, Host, Vault};
use anyhow::{Context, Result, bail};
use base64::Engine;
use base64::engine::general_purpose::STANDARD as B64;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use uuid::Uuid;

pub const GIST_FILENAME: &str = "rustmius-vault.enc";
const GIST_DESCRIPTION: &str = "Rustmius Encrypted Vault";
const GITHUB_API: &str = "https://api.github.com/gists";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub github_token: String,
    pub gist_id: String,
    pub auto_sync: bool,
    pub last_sync: Option<DateTime<Utc>>,
}

pub fn save_sync_config(path: &Path, cfg: &SyncConfig, master_password: &str) -> Result<()> {
    let json = serde_json::to_vec(cfg)?;
    let encrypted = encrypt_bytes(&json, master_password)?;
    write_secure(path, &encrypted)
}

pub fn load_sync_config(path: &Path, master_password: &str) -> Result<SyncConfig> {
    let data = fs::read(path)?;
    let plaintext = decrypt_bytes(&data, master_password)?;
    let cfg: SyncConfig = serde_json::from_slice(&plaintext)?;
    Ok(cfg)
}

pub fn encode_encrypted_vault(vault: &Vault, password: &str) -> Result<String> {
    let raw = encrypt_vault(vault, password)?;
    Ok(B64.encode(raw))
}

pub fn decode_encrypted_vault(encoded: &str, password: &str) -> Result<Vault> {
    let cleaned: String = encoded.chars().filter(|c| !c.is_whitespace()).collect();
    let raw = B64.decode(cleaned).context("Invalid Base64 Gist content")?;
    decrypt_vault(&raw, password)
}

/// Merges two vaults by `id`, keeping the entry with the most recent `updated_at`
/// while respecting deletion tombstones (`deleted_hosts`).
pub fn merge_vaults(local: &Vault, remote: &Vault) -> Vault {
    let mut deleted_hosts: HashMap<Uuid, DateTime<Utc>> = local.deleted_hosts.clone();
    for (&id, &ts) in &remote.deleted_hosts {
        deleted_hosts
            .entry(id)
            .and_modify(|existing| {
                if ts > *existing {
                    *existing = ts;
                }
            })
            .or_insert(ts);
    }

    let mut hosts_map: HashMap<Uuid, Host> = HashMap::new();
    for h in &local.hosts {
        hosts_map.insert(h.id, h.clone());
    }
    for h in &remote.hosts {
        hosts_map
            .entry(h.id)
            .and_modify(|existing| {
                if h.updated_at > existing.updated_at {
                    *existing = h.clone();
                }
            })
            .or_insert_with(|| h.clone());
    }
    hosts_map.retain(|id, host| {
        deleted_hosts
            .get(id)
            .map(|del_ts| host.updated_at > *del_ts)
            .unwrap_or(true)
    });

    let mut groups_map: HashMap<Uuid, Group> = HashMap::new();
    for g in &local.groups {
        groups_map.insert(g.id, g.clone());
    }
    for g in &remote.groups {
        groups_map
            .entry(g.id)
            .and_modify(|existing| {
                if g.updated_at > existing.updated_at {
                    *existing = g.clone();
                }
            })
            .or_insert_with(|| g.clone());
    }

    let mut known_hosts_map: HashMap<(String, u16), crate::storage::known_hosts::KnownHostEntry> =
        HashMap::new();
    for kh in local.known_hosts.iter().chain(remote.known_hosts.iter()) {
        let key = (kh.host.to_lowercase(), kh.port);
        known_hosts_map
            .entry(key)
            .and_modify(|existing| {
                if kh.added_at > existing.added_at {
                    *existing = kh.clone();
                }
            })
            .or_insert_with(|| kh.clone());
    }

    Vault {
        version: local.version.max(remote.version).max(1),
        hosts: hosts_map.into_values().collect(),
        groups: groups_map.into_values().collect(),
        deleted_hosts,
        known_hosts: known_hosts_map.into_values().collect(),
    }
}

fn github_client(token: &str) -> Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("Rustmius/3.0"),
    );
    headers.insert(
        reqwest::header::ACCEPT,
        reqwest::header::HeaderValue::from_static("application/vnd.github+json"),
    );
    let auth = format!("Bearer {}", token.trim());
    headers.insert(
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&auth).context("Invalid GitHub token")?,
    );
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(20))
        .build()?)
}

#[derive(Deserialize)]
struct GistFileResp {
    content: Option<String>,
}

#[derive(Deserialize)]
struct GistResp {
    id: String,
    files: HashMap<String, GistFileResp>,
}

pub async fn create_vault_gist(token: &str, encrypted_b64: &str) -> Result<String> {
    let client = github_client(token)?;
    let body = serde_json::json!({
        "description": GIST_DESCRIPTION,
        "public": false,
        "files": {
            GIST_FILENAME: {
                "content": encrypted_b64
            }
        }
    });

    let resp = client.post(GITHUB_API).json(&body).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("GitHub API ({status}): {text}");
    }
    let created: GistResp = resp.json().await?;
    Ok(created.id)
}

pub async fn find_existing_vault_gist(token: &str) -> Result<Option<(String, String)>> {
    let client = github_client(token)?;
    let resp = client
        .get(format!("{GITHUB_API}?per_page=100"))
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("GitHub API ({status}): {text}");
    }
    let gists: Vec<GistResp> = resp.json().await?;
    for g in gists {
        if g.files.contains_key(GIST_FILENAME) {
            let content = fetch_vault_gist(token, &g.id).await?;
            return Ok(Some((g.id, content)));
        }
    }
    Ok(None)
}

pub async fn fetch_vault_gist(token: &str, gist_id: &str) -> Result<String> {
    let client = github_client(token)?;
    let resp = client.get(format!("{GITHUB_API}/{gist_id}")).send().await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("GitHub API ({status}): {text}");
    }
    let gist: GistResp = resp.json().await?;
    let file = gist
        .files
        .get(GIST_FILENAME)
        .context("File rustmius-vault.enc not found in Gist")?;
    file.content.clone().context("Empty Gist content")
}

pub async fn update_vault_gist(token: &str, gist_id: &str, encrypted_b64: &str) -> Result<()> {
    let client = github_client(token)?;
    let body = serde_json::json!({
        "files": {
            GIST_FILENAME: {
                "content": encrypted_b64
            }
        }
    });
    let resp = client
        .patch(format!("{GITHUB_API}/{gist_id}"))
        .json(&body)
        .send()
        .await?;
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        bail!("GitHub API ({status}): {text}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn merges_vaults_keeping_newest_host() {
        let mut h1 = Host::new("web", "1.1.1.1", "root");
        let h1_id = h1.id;
        h1.updated_at = Utc::now() - Duration::minutes(10);

        let mut h1_newer = h1.clone();
        h1_newer.hostname = "2.2.2.2".to_string();
        h1_newer.updated_at = Utc::now();

        let h2 = Host::new("db", "3.3.3.3", "postgres");

        let local = Vault {
            version: 1,
            hosts: vec![h1],
            groups: vec![],
            deleted_hosts: HashMap::new(),
        };
        let remote = Vault {
            version: 1,
            hosts: vec![h1_newer, h2.clone()],
            groups: vec![],
            deleted_hosts: HashMap::new(),
        };

        let merged = merge_vaults(&local, &remote);
        assert_eq!(merged.hosts.len(), 2);
        let found = merged.hosts.iter().find(|h| h.id == h1_id).unwrap();
        assert_eq!(found.hostname, "2.2.2.2");
    }

    #[test]
    fn merges_vaults_respecting_deleted_host_tombstone() {
        let mut h1 = Host::new("old", "1.1.1.1", "root");
        h1.updated_at = Utc::now() - Duration::minutes(10);
        let h1_id = h1.id;

        let mut local = Vault::new();
        local.deleted_hosts.insert(h1_id, Utc::now());

        let mut remote = Vault::new();
        remote.hosts.push(h1);

        let merged = merge_vaults(&local, &remote);
        assert!(merged.hosts.is_empty());
        assert!(merged.deleted_hosts.contains_key(&h1_id));
    }

    #[test]
    fn encodes_and_decodes_base64_vault() {
        let mut v = Vault::new();
        v.hosts.push(Host::new("prod", "10.0.0.1", "admin"));
        let enc = encode_encrypted_vault(&v, "master-pass").unwrap();
        let dec = decode_encrypted_vault(&enc, "master-pass").unwrap();
        assert_eq!(dec.hosts.len(), 1);
        assert_eq!(dec.hosts[0].label, "prod");
    }
}
