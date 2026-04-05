use std::fs;
use directories::UserDirs;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
}

pub fn get_default_config_path() -> Option<std::path::PathBuf> {
    UserDirs::new().map(|dirs| dirs.home_dir().join(".ssh").join("config"))
}

pub fn load_hosts() -> Vec<SshHost> {
    if let Some(path) = get_default_config_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                return parse_ssh_config(&content);
            }
        }
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

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].to_lowercase();
        let value = parts[1];

        match key.as_str() {
            "host" => {
                if let Some(host) = current_host.take() {
                    if !host.alias.is_empty() && !host.hostname.is_empty() {
                        hosts.push(host);
                    }
                }
                current_host = Some(SshHost {
                    alias: value.to_string(),
                    hostname: String::new(),
                    user: None,
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
            _ => {}
        }
    }

    if let Some(host) = current_host {
        if !host.alias.is_empty() && !host.hostname.is_empty() {
            hosts.push(host);
        }
    }

    hosts
}

pub fn add_host_to_config(host: &SshHost) -> anyhow::Result<()> {
    let path = get_default_config_path().ok_or_else(|| anyhow::anyhow!("Could not find SSH config path"))?;
    
    // Ensure .ssh directory exists
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

    let entry = format!(
        "\nHost {}\n    HostName {}\n    User {}\n",
        host.alias,
        host.hostname,
        host.user.as_deref().unwrap_or("root")
    );

    content.push_str(&entry);
    std::fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_config_simple() {
        let config = "Host my-server\n  HostName 1.2.3.4\n  User root";
        let hosts = parse_ssh_config(config);
        assert_eq!(hosts.len(), 1);
        assert_eq!(hosts[0].alias, "my-server");
        assert_eq!(hosts[0].hostname, "1.2.3.4");
        assert_eq!(hosts[0].user, Some("root".to_string()));
    }

    #[test]
    fn test_parse_ssh_config_multiple() {
        let config = "
Host srv1
    HostName 10.0.0.1
Host srv2
    HostName 10.0.0.2
    User admin
";
        let hosts = parse_ssh_config(config);
        assert_eq!(hosts.len(), 2);
        assert_eq!(hosts[0].alias, "srv1");
        assert_eq!(hosts[1].alias, "srv2");
        assert_eq!(hosts[1].user, Some("admin".to_string()));
    }
}
