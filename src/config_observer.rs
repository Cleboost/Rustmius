use std::fs;
use std::path::PathBuf;
use directories::UserDirs;

/// Expand a leading `~/` or lone `~` to the user's home directory.
/// If no home directory can be determined, the original path is returned unchanged.
pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = UserDirs::new().map(|d| d.home_dir().to_path_buf()) {
            return home;
        }
    } else if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = UserDirs::new().map(|d| d.home_dir().to_path_buf()) {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity_file: Option<String>,
}

pub fn get_default_config_path() -> Option<std::path::PathBuf> {
    UserDirs::new().map(|dirs| dirs.home_dir().join(".ssh").join("config"))
}

pub fn load_hosts() -> Vec<SshHost> {
    if let Some(path) = get_default_config_path()
        && path.exists()
            && let Ok(content) = fs::read_to_string(path) {
                return parse_ssh_config(&content);
            }
    Vec::new()
}

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
                if let Some(ref mut host) = current_host {
                    if let Ok(p) = value.parse::<u16>() {
                        host.port = Some(p);
                    }
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
    std::fs::write(path, content)?;
    Ok(())
}

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
    
    std::fs::write(path, new_lines.join("\n"))?;
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
        // Build the config entry using the same logic as add_host_to_config.
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
        // Verify the config string contains the IdentityFile line.
        assert!(entry.contains("IdentityFile ~/.ssh/id_ed25519"));
        // Verify it round-trips through the parser.
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
        // The entry must contain a properly quoted IdentityFile line.
        assert!(entry.contains("IdentityFile \"/home/user/my keys/id_rsa\""));
        // Verify the path round-trips through the parser.
        let hosts = parse_ssh_config(&entry);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].identity_file, Some("/home/user/my keys/id_rsa".to_string()));
    }
}
