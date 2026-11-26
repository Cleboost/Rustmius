use crate::storage::{AuthMethod, Host, HostSource, Vault};
use std::fs;
use std::path::PathBuf;

pub fn default_ssh_config_path() -> Option<PathBuf> {
    directories::UserDirs::new().map(|u| u.home_dir().join(".ssh").join("config"))
}

pub fn parse_ssh_config(content: &str) -> Vec<Host> {
    let mut hosts = Vec::new();
    let mut current: Vec<Host> = Vec::new();

    let flush = |current: &mut Vec<Host>, hosts: &mut Vec<Host>| {
        for h in current.drain(..) {
            if !h.hostname.is_empty() {
                hosts.push(h);
            }
        }
    };

    for raw_line in content.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }

        let mut parts = line.splitn(2, |c: char| c.is_whitespace() || c == '=');
        let key = parts.next().unwrap_or("").trim().to_ascii_lowercase();
        let raw_val = parts.next().unwrap_or("").trim();
        let val = raw_val.trim_matches('"');

        if key == "host" {
            flush(&mut current, &mut hosts);
            let default_user = whoami_fallback();
            for token in raw_val.split_whitespace() {
                let alias = token.trim_matches('"');
                // Ignore wildcards (`*`, `?`) and negated patterns (`!pattern`)
                if alias.is_empty()
                    || alias.contains('*')
                    || alias.contains('?')
                    || alias.starts_with('!')
                {
                    continue;
                }
                let mut h = Host::new(alias, alias, default_user.clone());
                h.source = HostSource::SshConfig;
                current.push(h);
            }
        } else if key == "match" {
            // Conditional Match blocks are not evaluated; flush previous Host so
            // Match directives don't leak into it.
            flush(&mut current, &mut hosts);
        } else if !current.is_empty() {
            match key.as_str() {
                "hostname" => {
                    for h in &mut current {
                        h.hostname = val.to_string();
                    }
                }
                "user" => {
                    for h in &mut current {
                        h.username = val.to_string();
                    }
                }
                "port" => {
                    if let Ok(p) = val.parse::<u16>() {
                        for h in &mut current {
                            h.port = p;
                        }
                    }
                }
                "identityfile" => {
                    for h in &mut current {
                        h.auth = AuthMethod::PrivateKey {
                            path: val.to_string(),
                        };
                    }
                }
                _ => {}
            }
        }
    }

    flush(&mut current, &mut hosts);

    hosts
}

fn whoami_fallback() -> String {
    std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "root".to_string())
}

/// Filters parsed SSH config hosts to those whose alias (`label`) is not yet in `vault`.
pub fn filter_unimported_hosts(parsed: Vec<Host>, vault: &Vault) -> Vec<Host> {
    parsed
        .into_iter()
        .filter(|host| !vault.hosts.iter().any(|existing| existing.label == host.label))
        .collect()
}

/// Returns hosts from `~/.ssh/config` that are not yet in the `Vault`.
pub fn find_unimported_ssh_hosts(vault: &Vault) -> Vec<Host> {
    let Some(path) = default_ssh_config_path() else {
        return Vec::new();
    };
    let Ok(content) = fs::read_to_string(path) else {
        return Vec::new();
    };

    filter_unimported_hosts(parse_ssh_config(&content), vault)
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

    #[test]
    fn parses_multi_alias_and_ignores_match_block() {
        let cfg = r#"
Host a b *.ignored !excluded
    HostName 10.0.0.5
    User admin
    Port 2200

Match host 10.0.0.5 exec "true"
    User match_user
    Port 9999
"#;

        let hosts = parse_ssh_config(cfg);
        assert_eq!(hosts.len(), 2);

        assert_eq!(hosts[0].label, "a");
        assert_eq!(hosts[0].hostname, "10.0.0.5");
        assert_eq!(hosts[0].username, "admin");
        assert_eq!(hosts[0].port, 2200);

        assert_eq!(hosts[1].label, "b");
        assert_eq!(hosts[1].hostname, "10.0.0.5");
        assert_eq!(hosts[1].username, "admin");
        assert_eq!(hosts[1].port, 2200);
    }

    #[test]
    fn keeps_distinct_aliases_for_same_hostname_and_port() {
        let cfg = r#"
Host rm-alpine
    HostName 127.0.0.1
    User root
    Port 2222

Host "rm-quoted" rm-multi
    HostName 127.0.0.1
    User root
    Port 2222
"#;

        let parsed = parse_ssh_config(cfg);
        assert_eq!(parsed.len(), 3);

        let mut vault = Vault::default();
        let mut imported = Host::new("rm-alpine", "127.0.0.1", "root");
        imported.port = 2222;
        vault.hosts.push(imported);

        let unimported = filter_unimported_hosts(parsed, &vault);
        let labels: Vec<&str> = unimported.iter().map(|h| h.label.as_str()).collect();
        assert_eq!(labels, vec!["rm-quoted", "rm-multi"]);
    }
}
