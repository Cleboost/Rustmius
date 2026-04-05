use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SshHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
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
