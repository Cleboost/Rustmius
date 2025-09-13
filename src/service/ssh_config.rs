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

pub fn delete_ssh_server(server_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME")?;
    let config_path = Path::new(&home_dir).join(".ssh").join("config");

    if !config_path.exists() {
        return Err("SSH config file does not exist".into());
    }

    let content = fs::read_to_string(&config_path)?;
    let new_content = remove_server_from_config(&content, server_name)?;

    fs::write(&config_path, new_content)?;
    Ok(())
}

fn remove_server_from_config(
    content: &str,
    server_name: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let lines: Vec<&str> = content.lines().collect();
    let mut new_lines = Vec::new();
    let mut skip_section = false;
    let mut in_host_section = false;

    for line in lines {
        let trimmed_line = line.trim();

        if trimmed_line.to_lowercase().starts_with("host ") {
            let host_name = trimmed_line[5..].trim();

            if host_name == server_name {
                skip_section = true;
                in_host_section = true;
                continue;
            } else {
                skip_section = false;
                in_host_section = true;
            }
        } else if trimmed_line.is_empty() && in_host_section {
            if skip_section {
                skip_section = false;
                in_host_section = false;
                continue;
            }
            in_host_section = false;
        } else if !trimmed_line.is_empty() && !trimmed_line.starts_with('#') && in_host_section {
            if skip_section {
                continue;
            }
        }

        if !skip_section {
            new_lines.push(line);
        }
    }

    Ok(new_lines.join("\n"))
}

pub fn export_ssh_config_to_file(servers: &[SshServer], file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_content = String::new();
    
    for server in servers {
        config_content.push_str(&format!("Host {}\n", server.name));
        
        if let Some(ref hostname) = server.hostname {
            config_content.push_str(&format!("    HostName {}\n", hostname));
        }
        
        if let Some(ref user) = server.user {
            config_content.push_str(&format!("    User {}\n", user));
        }
        
        if let Some(ref identity_file) = server.identity_file {
            config_content.push_str(&format!("    IdentityFile {}\n", identity_file));
        }
        
        if let Some(port) = server.port {
            config_content.push_str(&format!("    Port {}\n", port));
        }
        
        config_content.push('\n');
    }
    
    fs::write(file_path, config_content)?;
    Ok(())
}
