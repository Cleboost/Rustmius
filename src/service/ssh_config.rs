use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SshServer {
    pub name: String,
    pub hostname: Option<String>,
    pub user: Option<String>,
    pub identity_file: Option<String>,
    pub port: Option<u16>,
}

pub fn load_ssh_servers() -> Result<Vec<SshServer>, Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME")?;
    let config_path = Path::new(&home_dir).join(".ssh").join("config");

    if !config_path.exists() {
        return Ok(vec![]);
    }

    let content = fs::read_to_string(&config_path)?;
    parse_ssh_config(&content)
}

fn parse_ssh_config(content: &str) -> Result<Vec<SshServer>, Box<dyn std::error::Error>> {
    let mut servers = Vec::new();
    let mut current_server: Option<SshServer> = None;

    for line in content.lines() {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.to_lowercase().starts_with("host ") {
            if let Some(server) = current_server.take() {
                servers.push(server);
            }

            let host_name = line[5..].trim().to_string();
            current_server = Some(SshServer {
                name: host_name,
                hostname: None,
                user: None,
                identity_file: None,
                port: None,
            });
        } else if let Some(ref mut server) = current_server {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let directive = parts[0].to_lowercase();
                let value = parts[1].to_string();

                match directive.as_str() {
                    "hostname" => server.hostname = Some(value),
                    "user" => server.user = Some(value),
                    "identityfile" => server.identity_file = Some(value),
                    "port" => {
                        if let Ok(port) = value.parse::<u16>() {
                            server.port = Some(port);
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(server) = current_server {
        servers.push(server);
    }

    Ok(servers)
}