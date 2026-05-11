use std::fs;
use std::path::PathBuf;
use directories::UserDirs;
use anyhow::Context;
use std::sync::{RwLock, OnceLock};

static APP_CONFIG_CACHE: OnceLock<RwLock<AppConfig>> = OnceLock::new();
static HOSTS_CACHE: OnceLock<RwLock<Vec<SshHost>>> = OnceLock::new();
static HOME_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

fn get_home_dir() -> Option<PathBuf> {
    HOME_DIR.get_or_init(|| UserDirs::new().map(|d| d.home_dir().to_path_buf())).clone()
}

/// Expands a tilde (`~`) at the beginning of a path string into the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = get_home_dir() {
            return home;
        }
    } else if let Some(rest) = path.strip_prefix("~/")
        && let Some(home) = get_home_dir() {
            return home.join(rest);
        }
    PathBuf::from(path)
}

/// Returns the path to the user's `.ssh` directory.
pub fn get_ssh_dir() -> Option<PathBuf> {
    get_home_dir().map(|h| h.join(".ssh"))
}

pub const REMOTE_SSH_DIR: &str = "~/.ssh";
pub const REMOTE_AUTHORIZED_KEYS: &str = "~/.ssh/authorized_keys";
#[allow(dead_code)]
pub const REMOTE_SSH_CONFIG: &str = "~/.ssh/config";

/// Represents an SSH public/private key pair.
#[derive(Debug, Clone)]
pub struct SshKeyPair {
    pub name: String,
    pub pub_path: PathBuf,
    pub priv_path: PathBuf,
}

/// Scans the local `.ssh` directory for public keys with corresponding private keys.
pub fn load_ssh_keys() -> anyhow::Result<Vec<SshKeyPair>> {
    let ssh_dir = get_ssh_dir().ok_or_else(|| anyhow::anyhow!("Could not determine SSH directory"))?;
    if !ssh_dir.exists() {
        return Ok(Vec::new());
    }

    let mut keys: Vec<SshKeyPair> = fs::read_dir(&ssh_dir)
        .context("Failed to read SSH directory")?
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pub") {
                let mut priv_path = path.clone();
                priv_path.set_extension("");
                if priv_path.exists() {
                    let name = path.file_stem()?.to_string_lossy().to_string();
                    return Some(SshKeyPair {
                        name,
                        pub_path: path,
                        priv_path,
                    });
                }
            }
            None
        })
        .collect();

    keys.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(keys)
}

/// Retrieves a password from the system keyring for a given server alias.
pub async fn get_keyring_password(alias: &str) -> Option<String> {
    if let Ok(keyring) = oo7::Keyring::new().await {
        let mut attr = std::collections::HashMap::new();
        let alias_lower = alias.to_lowercase();
        attr.insert("rustmius-server-alias", alias_lower.as_str());
        if let Ok(items) = keyring.search_items(&attr).await {
            if let Some(item) = items.first() {
                if let Ok(secret) = item.secret().await {
                    return std::str::from_utf8(&secret).map(String::from).ok();
                }
            }
        }
    }
    None
}

/// Global application configuration settings.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AppConfig {
    pub monitor_refresh_rate: u32, // index: 0=1s, 1=3s, 2=5s, 3=10s
    pub terminal_font: String,
    pub terminal_scrollback: u32,
    pub confirm_tab_close: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            monitor_refresh_rate: 1, // 3s
            terminal_font: "Monospace 11".to_string(),
            terminal_scrollback: 10000,
            confirm_tab_close: false,
        }
    }
}

pub fn get_app_config_path() -> Option<PathBuf> {
    directories::ProjectDirs::from("org", "rustmius", "Rustmius")
        .map(|dirs| dirs.config_dir().join("config.json"))
}

/// Loads the application configuration, using a cached version if available.
pub fn load_app_config() -> anyhow::Result<AppConfig> {
    if let Some(cache) = APP_CONFIG_CACHE.get() {
        return Ok(cache.read().map_err(|_| anyhow::anyhow!("Cache lock poisoned"))?.clone());
    }
    refresh_app_config()
}

/// Forces a reload of the application configuration from disk and updates the cache.
pub fn refresh_app_config() -> anyhow::Result<AppConfig> {
    let path = get_app_config_path().ok_or_else(|| anyhow::anyhow!("Could not determine app config path"))?;
    let config = if !path.exists() {
        AppConfig::default()
    } else {
        let content = fs::read_to_string(&path).context("Failed to read app config file")?;
        serde_json::from_str(&content).context("Failed to parse app config JSON")?
    };

    if let Some(cache) = APP_CONFIG_CACHE.get() {
        let mut guard = cache.write().map_err(|_| anyhow::anyhow!("Cache lock poisoned"))?;
        *guard = config.clone();
    } else {
        let _ = APP_CONFIG_CACHE.get_or_init(|| RwLock::new(config.clone()));
    }
    Ok(config)
}

/// Saves the provided application configuration to disk and updates the cache.
pub fn save_app_config(config: &AppConfig) -> anyhow::Result<()> {
    let path = get_app_config_path().ok_or_else(|| anyhow::anyhow!("Could not find app config path"))?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;

    if let Some(cache) = APP_CONFIG_CACHE.get() {
        let mut guard = cache.write().map_err(|_| anyhow::anyhow!("Cache lock poisoned"))?;
        *guard = config.clone();
    } else {
        let _ = APP_CONFIG_CACHE.get_or_init(|| RwLock::new(config.clone()));
    }
    Ok(())
}

/// Represents an SSH host entry as defined in an SSH config file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
}

pub fn get_default_config_path() -> Option<std::path::PathBuf> {
    get_ssh_dir().map(|d| d.join("config"))
}

/// Loads the list of SSH hosts from the default SSH config file, using a cached version if available.
pub fn load_hosts() -> anyhow::Result<Vec<SshHost>> {
    if let Some(cache) = HOSTS_CACHE.get() {
        return Ok(cache.read().map_err(|_| anyhow::anyhow!("Cache lock poisoned"))?.clone());
    }
    refresh_hosts()
}

/// Forces a reload of the SSH hosts from the config file and updates the cache.
pub fn refresh_hosts() -> anyhow::Result<Vec<SshHost>> {
    let path = get_default_config_path().ok_or_else(|| anyhow::anyhow!("Could not determine SSH config path"))?;
    let hosts = if !path.exists() {
        Vec::new()
    } else {
        let content = fs::read_to_string(&path).context("Failed to read SSH config file")?;
        parse_ssh_config(&content)
    };

    if let Some(cache) = HOSTS_CACHE.get() {
        let mut guard = cache.write().map_err(|_| anyhow::anyhow!("Cache lock poisoned"))?;
        *guard = hosts.clone();
    } else {
        let _ = HOSTS_CACHE.get_or_init(|| RwLock::new(hosts.clone()));
    }
    Ok(hosts)
}

/// Parses a raw SSH configuration string into a vector of `SshHost` structures.
pub fn parse_ssh_config(content: &str) -> Vec<SshHost> {
    let mut hosts = Vec::new();
    let mut current_host: Option<SshHost> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.splitn(2, |c: char| c.is_whitespace()).collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].to_lowercase();
        let mut value = parts[1].trim();

        if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
            value = &value[1..value.len() - 1];
        }

        match key.as_str() {
            "host" => {
                if let Some(host) = current_host.take()
                    && !host.alias.is_empty() && !host.hostname.is_empty() {
                        hosts.push(host);
                    }
                current_host = Some(SshHost {
                    alias: value.to_string(),
                    hostname: String::new(),
                    user: None,
                    port: None,
                    identity_file: None,
                });
            }
            "hostname" => {
                if let Some(ref mut host) = current_host {
                    host.hostname = value.to_string();
                }
            }
            "user" => {
                if let Some(ref mut host) = current_host {
                    host.user = Some(value.to_string());
                }
            }
            "port" => {
                if let Some(ref mut host) = current_host
                    && let Ok(p) = value.parse::<u16>() {
                        host.port = Some(p);
                    }
            }
            "identityfile" => {
                if let Some(ref mut host) = current_host {
                    host.identity_file = Some(value.to_string());
                }
            }
            _ => {}
        }
    }

    if let Some(host) = current_host
        && !host.alias.is_empty() && !host.hostname.is_empty() {
            hosts.push(host);
        }

    hosts
}

/// Appends a new SSH host entry to the user's SSH config file and updates the cache.
pub fn add_host_to_config(host: &SshHost) -> anyhow::Result<()> {
    let path = get_default_config_path().ok_or_else(|| anyhow::anyhow!("Could not find SSH config path"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    let alias_quoted = if host.alias.contains(' ') {
        format!("\"{}\"", host.alias)
    } else {
        host.alias.clone()
    };

    let mut entry = format!(
        "\nHost {}\n    HostName {}\n    User {}\n    Port {}\n",
        alias_quoted,
        host.hostname,
        host.user.as_deref().unwrap_or("root"),
        host.port.unwrap_or(22)
    );

    if let Some(ref id_file) = host.identity_file {
        let id_file_quoted = if id_file.contains(' ') {
            format!("\"{}\"", id_file)
        } else {
            id_file.clone()
        };
        entry.push_str(&format!("    IdentityFile {}\n", id_file_quoted));
    }

    content.push_str(&entry);
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, &content)?;
    std::fs::rename(tmp_path, path)?;

    if let Some(cache) = HOSTS_CACHE.get() {
        let mut guard = cache.write().map_err(|_| anyhow::anyhow!("Cache lock poisoned"))?;
        *guard = parse_ssh_config(&content);
    }
    Ok(())
}

/// Removes an SSH host entry from the config file by its alias and updates the cache.
pub fn delete_host_from_config(alias: &str) -> anyhow::Result<()> {
    let path = get_default_config_path().ok_or_else(|| anyhow::anyhow!("No config path"))?;
    if !path.exists() { return Ok(()); }
    let content = std::fs::read_to_string(&path)?;
    let mut new_lines = Vec::new();
    let mut skip = false;
    let target_alias = alias.to_lowercase();

    for line in content.lines() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("host ") {
            let mut val = trimmed["host ".len()..].trim();
            if val.starts_with('"') && val.ends_with('"') && val.len() >= 2 {
                val = &val[1..val.len()-1];
            }
            if val == target_alias {
                skip = true;
                continue;
            } else {
                skip = false;
            }
        }
        if skip && (line.starts_with(' ') || line.starts_with('\t') || line.trim().is_empty()) {
            continue;
        }
        if skip {
            skip = false;
        }
        new_lines.push(line);
    }
    let new_content = new_lines.join("\n");
    let tmp_path = path.with_extension("tmp");
    std::fs::write(&tmp_path, &new_content)?;
    std::fs::rename(tmp_path, path)?;

    if let Some(cache) = HOSTS_CACHE.get() {
        let mut guard = cache.write().map_err(|_| anyhow::anyhow!("Cache lock poisoned"))?;
        *guard = parse_ssh_config(&new_content);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_config_simple() {
        let config = "Host my-server\n  HostName 1.2.3.4\n  User root\n  Port 2222";
        let hosts = parse_ssh_config(config);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "my-server");
        assert_eq!(hosts[0].hostname, "1.2.3.4");
        assert_eq!(hosts[0].user, Some("root".to_string()));
        assert_eq!(hosts[0].port, Some(2222));
    }

    #[test]
    fn test_parse_ssh_config_with_spaces() {
        let config = "Host \"My Server\"\n  HostName 1.2.3.4\n  User root";
        let hosts = parse_ssh_config(config);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "My Server");
    }

    #[test]
    fn test_parse_ssh_config_with_identity_file() {
        let config = "Host my-server\n  HostName 1.2.3.4\n  User root\n  Port 22\n  IdentityFile ~/.ssh/id_ed25519";
        let hosts = parse_ssh_config(config);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].identity_file, Some("~/.ssh/id_ed25519".to_string()));
    }

    #[test]
    fn test_parse_ssh_config_with_quoted_identity_file() {
        let config = "Host my-server\n  HostName 1.2.3.4\n  User root\n  IdentityFile \"/home/user/my keys/id_ed25519\"";
        let hosts = parse_ssh_config(config);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].identity_file, Some("/home/user/my keys/id_ed25519".to_string()));
    }

    #[test]
    fn test_add_host_to_config_emits_identity_file() {
        let host = SshHost {
            alias: "test-host".to_string(),
            hostname: "192.168.1.1".to_string(),
            user: Some("admin".to_string()),
            port: Some(22),
            identity_file: Some("~/.ssh/id_ed25519".to_string()),
        };
        let alias_quoted = if host.alias.contains(' ') {
            format!("\"{}\"", host.alias)
        } else {
            host.alias.clone()
        };
        let mut entry = format!(
            "\nHost {}\n    HostName {}\n    User {}\n    Port {}\n",
            alias_quoted,
            host.hostname,
            host.user.as_deref().unwrap_or("root"),
            host.port.unwrap_or(22)
        );
        if let Some(ref id_file) = host.identity_file {
            let id_file_quoted = if id_file.contains(' ') {
                format!("\"{}\"", id_file)
            } else {
                id_file.clone()
            };
            entry.push_str(&format!("    IdentityFile {}\n", id_file_quoted));
        }
        assert!(entry.contains("IdentityFile ~/.ssh/id_ed25519"));
        let hosts = parse_ssh_config(&entry);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].identity_file, Some("~/.ssh/id_ed25519".to_string()));
    }

    #[test]
    fn test_add_host_to_config_quotes_identity_file_with_spaces() {
        let host = SshHost {
            alias: "spaced-host".to_string(),
            hostname: "10.0.0.1".to_string(),
            user: Some("user".to_string()),
            port: Some(22),
            identity_file: Some("/home/user/my keys/id_rsa".to_string()),
        };
        let mut entry = format!(
            "\nHost {}\n    HostName {}\n    User {}\n    Port {}\n",
            host.alias, host.hostname,
            host.user.as_deref().unwrap_or("root"),
            host.port.unwrap_or(22)
        );
        if let Some(ref id_file) = host.identity_file {
            let id_file_quoted = if id_file.contains(' ') {
                format!("\"{}\"", id_file)
            } else {
                id_file.clone()
            };
            entry.push_str(&format!("    IdentityFile {}\n", id_file_quoted));
        }
        assert!(entry.contains("IdentityFile \"/home/user/my keys/id_rsa\""));
        let hosts = parse_ssh_config(&entry);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].identity_file, Some("/home/user/my keys/id_rsa".to_string()));
    }
}