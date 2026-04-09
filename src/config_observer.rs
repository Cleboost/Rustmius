use std::fs;
use directories::UserDirs;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
    pub port: Option<u16>,
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

    let entry = format!(
        "\nHost {}\n    HostName {}\n    User {}\n    Port {}\n",
        alias_quoted,
        host.hostname,
        host.user.as_deref().unwrap_or("root"),
        host.port.unwrap_or(22)
    );

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
}
