use crate::storage::{AuthMethod, Host, HostSource, Vault};
use std::fs;
use std::path::PathBuf;

pub fn default_ssh_config_path() -> Option<PathBuf> {
    directories::UserDirs::new().map(|u| u.home_dir().join(".ssh").join("config"))
}

pub fn parse_ssh_config(content: &str) -> Vec<Host> {
    let mut hosts = Vec::new();
    let mut current: Option<Host> = None;

    for raw_line in content.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(2, |c: char| c.is_whitespace() || c == '=');
        let key = parts.next().unwrap_or("").trim().to_ascii_lowercase();
        let val = parts.next().unwrap_or("").trim().trim_matches('"');

        if key == "host" {
            if let Some(h) = current.take()
                && !h.hostname.is_empty() {
                    hosts.push(h);
                }
            // Ignore wildcards like `Host *` or `Host *.example.com`
            let alias = val.split_whitespace().next().unwrap_or("");
            if alias.is_empty() || alias.contains('*') || alias.contains('?') {
                current = None;
            } else {
                let mut h = Host::new(alias, alias, whoami_fallback());
                h.source = HostSource::SshConfig;
                current = Some(h);
            }
        } else if let Some(ref mut h) = current {
            match key.as_str() {
                "hostname" => h.hostname = val.to_string(),
                "user" => h.username = val.to_string(),
                "port" => {
                    if let Ok(p) = val.parse::<u16>() {
                        h.port = p;
                    }
                }
                "identityfile" => {
                    h.auth = AuthMethod::PrivateKey {
                        path: val.to_string(),
                    };
                }
                _ => {}
            }
        }
    }

    if let Some(h) = current.take()
        && !h.hostname.is_empty() {
            hosts.push(h);
        }

    hosts
}

fn whoami_fallback() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "root".to_string())
}

/// Returns hosts from `~/.ssh/config` that are not yet in the `Vault`.
pub fn find_unimported_ssh_hosts(vault: &Vault) -> Vec<Host> {
    let Some(path) = default_ssh_config_path() else {
        return Vec::new();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    parse_ssh_config(&content)
        .into_iter()
        .filter(|host| {
            !vault.hosts.iter().any(|existing| {
                existing.label == host.label
                    || (existing.hostname == host.hostname
                        && existing.username == host.username
                        && existing.port == host.port)
            })
        })
        .collect()
}

/// Imports hosts from `~/.ssh/config` that are not yet in the `Vault`.
/// Returns the number of added hosts.
pub fn import_ssh_config_into_vault(vault: &mut Vault) -> usize {
    let to_add = find_unimported_ssh_hosts(vault);
    let added = to_add.len();
    vault.hosts.extend(to_add);
    added
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_standard_ssh_config() {
        let cfg = r#"
# Global defaults
Host *
    ServerAliveInterval 60

Host prod-api
    HostName 192.168.1.50
    User deploy
    Port 2222
    IdentityFile ~/.ssh/id_ed25519

Host staging
    HostName staging.example.com
    User ubuntu
"#;

        let hosts = parse_ssh_config(cfg);
        assert_eq!(hosts.len(), 2);

        assert_eq!(hosts[0].label, "prod-api");
        assert_eq!(hosts[0].hostname, "192.168.1.50");
        assert_eq!(hosts[0].username, "deploy");
        assert_eq!(hosts[0].port, 2222);
        assert_eq!(
            hosts[0].auth,
            AuthMethod::PrivateKey {
                path: "~/.ssh/id_ed25519".to_string()
            }
        );

        assert_eq!(hosts[1].label, "staging");
        assert_eq!(hosts[1].hostname, "staging.example.com");
        assert_eq!(hosts[1].username, "ubuntu");
        assert_eq!(hosts[1].port, 22);
    }
}
