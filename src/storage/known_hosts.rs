use anyhow::{Context, Result};
use russh::keys::{HashAlg, PublicKey, PublicKeyBase64, PublicKeyOrCertificate};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KnownHostEntry {
    pub host: String,
    pub port: u16,
    pub algorithm: String,
    pub public_key_b64: String,
    pub fingerprint_sha256: String,
    pub added_at: u64,
}

#[derive(Debug, Clone)]
pub enum HostKeyCheckResult {
    Trusted,
    Unknown,
    Changed {
        old_fingerprint: String,
        old_algorithm: String,
    },
}

pub fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn os_known_hosts_path() -> Option<PathBuf> {
    directories::UserDirs::new().map(|d| d.home_dir().join(".ssh").join("known_hosts"))
}

pub fn format_host_pattern(host: &str, port: u16) -> String {
    if port == 22 {
        host.to_string()
    } else {
        format!("[{host}]:{port}")
    }
}

pub fn extract_key_info(key: &PublicKeyOrCertificate) -> (String, String, String) {
    match key {
        PublicKeyOrCertificate::PublicKey { key: pk, .. } => (
            pk.algorithm().to_string(),
            pk.public_key_base64(),
            pk.fingerprint(HashAlg::Sha256).to_string(),
        ),
        PublicKeyOrCertificate::Certificate(cert) => {
            let pk = PublicKey::new(cert.public_key().clone(), "");
            (
                pk.algorithm().to_string(),
                pk.public_key_base64(),
                pk.fingerprint(HashAlg::Sha256).to_string(),
            )
        }
    }
}

fn parse_host_token(token: &str) -> Option<(String, u16)> {
    if token.starts_with('|') {
        // Hashed known_hosts entry; skip for plain-text listing.
        return None;
    }
    if let Some(rest) = token.strip_prefix('[')
        && let Some((host, port_str)) = rest.split_once("]:")
        && let Ok(port) = port_str.parse::<u16>()
    {
        return Some((host.to_string(), port));
    }
    Some((token.to_string(), 22))
}

fn line_matches_host(line: &str, host: &str, port: u16) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('@') {
        return false;
    }
    let Some(hosts_part) = trimmed.split_whitespace().next() else {
        return false;
    };
    hosts_part
        .split(',')
        .filter_map(parse_host_token)
        .any(|(h, p)| h.eq_ignore_ascii_case(host) && p == port)
}

pub fn load_os_known_hosts() -> Vec<KnownHostEntry> {
    let Some(path) = os_known_hosts_path() else {
        return Vec::new();
    };
    let Ok(content) = std::fs::read_to_string(path) else {
        return Vec::new();
    };

    let mut entries = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') || trimmed.starts_with('@') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let (Some(hosts_part), Some(algo), Some(key_b64)) =
            (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };

        let fingerprint_sha256 = PublicKey::from_openssh(&format!("{algo} {key_b64}"))
            .map(|pk| pk.fingerprint(HashAlg::Sha256).to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        for token in hosts_part.split(',') {
            if let Some((host, port)) = parse_host_token(token) {
                entries.push(KnownHostEntry {
                    host,
                    port,
                    algorithm: algo.to_string(),
                    public_key_b64: key_b64.to_string(),
                    fingerprint_sha256: fingerprint_sha256.clone(),
                    added_at: 0,
                });
            }
        }
    }
    entries
}

pub fn verify_host_key(
    host: &str,
    port: u16,
    key: &PublicKeyOrCertificate,
    vault_hosts: &[KnownHostEntry],
) -> HostKeyCheckResult {
    let (_algo, key_b64, _fp) = extract_key_info(key);

    // Check vault first, then OS ~/.ssh/known_hosts.
    if let Some(entry) = vault_hosts
        .iter()
        .find(|e| e.host.eq_ignore_ascii_case(host) && e.port == port)
    {
        if entry.public_key_b64 == key_b64 {
            return HostKeyCheckResult::Trusted;
        }
        return HostKeyCheckResult::Changed {
            old_fingerprint: entry.fingerprint_sha256.clone(),
            old_algorithm: entry.algorithm.clone(),
        };
    }

    let os_hosts = load_os_known_hosts();
    if let Some(entry) = os_hosts
        .iter()
        .find(|e| e.host.eq_ignore_ascii_case(host) && e.port == port)
    {
        if entry.public_key_b64 == key_b64 {
            return HostKeyCheckResult::Trusted;
        }
        return HostKeyCheckResult::Changed {
            old_fingerprint: entry.fingerprint_sha256.clone(),
            old_algorithm: entry.algorithm.clone(),
        };
    }

    HostKeyCheckResult::Unknown
}

pub fn upsert_os_known_host(
    host: &str,
    port: u16,
    algorithm: &str,
    public_key_b64: &str,
) -> Result<()> {
    let path = os_known_hosts_path().context("Could not determine home directory")?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).context("Failed to create ~/.ssh directory")?;
    }

    let mut lines: Vec<String> = if path.exists() {
        std::fs::read_to_string(&path)
            .unwrap_or_default()
            .lines()
            .filter(|l| !line_matches_host(l, host, port))
            .map(|l| l.to_string())
            .collect()
    } else {
        Vec::new()
    };

    let pattern = format_host_pattern(host, port);
    lines.push(format!("{pattern} {algorithm} {public_key_b64}"));

    let mut out = lines.join("\n");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(&path, out).context("Failed to write ~/.ssh/known_hosts")?;
    Ok(())
}

pub fn remove_os_known_host(host: &str, port: u16) -> Result<()> {
    let Some(path) = os_known_hosts_path() else {
        return Ok(());
    };
    if !path.exists() {
        return Ok(());
    }
    let lines: Vec<String> = std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !line_matches_host(l, host, port))
        .map(|l| l.to_string())
        .collect();

    let mut out = lines.join("\n");
    if !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    std::fs::write(&path, out).context("Failed to update ~/.ssh/known_hosts")?;
    Ok(())
}
