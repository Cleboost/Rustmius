use std::collections::HashSet;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SshServer {
    pub name: String,
    pub display_name: Option<String>,
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
    let cleaned_content = clean_ssh_config(&content);

    if cleaned_content != content {
        fs::write(&config_path, &cleaned_content)?;
    }

    parse_ssh_config(&cleaned_content)
}

fn clean_ssh_config(content: &str) -> String {
    let lines: Vec<&str> = content.lines().collect();
    let mut cleaned_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        if !trimmed.to_lowercase().starts_with("displayname") {
            cleaned_lines.push(line);
        }
    }

    cleaned_lines.join("\n")
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

            let host_args = line[5..].trim();
            let tokens: Vec<&str> = host_args.split_whitespace().collect();
            let host_name = tokens.get(0).unwrap_or(&"").to_string();
            let display_name = if tokens.len() > 1 {
                Some(tokens[1..].join(" "))
            } else {
                None
            };

            current_server = Some(SshServer {
                name: host_name,
                display_name,
                hostname: None,
                user: None,
                identity_file: None,
                port: None,
            });
        } else if let Some(ref mut server) = current_server {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let directive = parts[0].to_lowercase();
                let value = parts[1..].join(" ");

                match directive.as_str() {
                    "hostname" => server.hostname = Some(value),
                    "user" => server.user = Some(value),
                    "identityfile" => server.identity_file = Some(value),
                    "port" => {
                        if let Ok(port) = value.parse::<u16>() {
                            server.port = Some(port);
                        }
                    }
                    "displayname" => server.display_name = Some(value),
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

pub fn export_ssh_config_to_file(
    servers: &[SshServer],
    file_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut config_content = String::new();

    for server in servers {
        config_content.push_str(&format!("Host {}\n", server.name));

        if let Some(ref display) = server.display_name {
            config_content.push_str(&format!("    DisplayName {}\n", display));
        }

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

pub fn import_ssh_config_from_file(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let home_dir = std::env::var("HOME")?;
    let ssh_dir = Path::new(&home_dir).join(".ssh");
    let config_path = ssh_dir.join("config");

    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir)?;
    }

    let import_content = fs::read_to_string(file_path)?;

    if !is_valid_ssh_config(&import_content) {
        return Err("The selected file does not seem to be a valid SSH configuration file".into());
    }

    if config_path.exists() {
        let backup_path = ssh_dir.join("config.backup");
        fs::copy(&config_path, &backup_path)?;
    }

    fs::write(&config_path, import_content)?;

    Ok(())
}

fn is_valid_ssh_config(content: &str) -> bool {
    let lines: Vec<&str> = content.lines().collect();
    let mut has_host_directive = false;
    let mut has_valid_directives = false;

    for line in lines {
        let line = line.trim();

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if line.to_lowercase().starts_with("host ") {
            has_host_directive = true;
        } else if has_host_directive {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let directive = parts[0].to_lowercase();
                match directive.as_str() {
                    "hostname"
                    | "user"
                    | "identityfile"
                    | "port"
                    | "proxycommand"
                    | "forwardagent"
                    | "compression"
                    | "serveraliveinterval"
                    | "serveralivecountmax" => {
                        has_valid_directives = true;
                    }
                    _ => {}
                }
            }
        }
    }

    has_host_directive && has_valid_directives
}

pub fn generate_valid_hostname(
    display_name: &str,
    existing_servers: &[SshServer],
) -> (String, String) {
    let clean_name = display_name
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>()
        .to_lowercase();

    if clean_name.is_empty()
        || display_name.contains(char::is_whitespace)
        || display_name
            .chars()
            .any(|c| !c.is_alphanumeric() && c != '-' && c != '_')
    {
        let random_name = generate_random_hostname(existing_servers);
        (random_name, display_name.to_string())
    } else {
        let existing_names: HashSet<String> =
            existing_servers.iter().map(|s| s.name.clone()).collect();

        if existing_names.contains(&clean_name) {
            let random_name = generate_random_hostname(existing_servers);
            (random_name, display_name.to_string())
        } else {
            (clean_name, display_name.to_string())
        }
    }
}

fn generate_random_hostname(existing_servers: &[SshServer]) -> String {
    use rand::Rng;
    use std::collections::HashSet;

    let existing_names: HashSet<String> = existing_servers.iter().map(|s| s.name.clone()).collect();

    let mut rng = rand::thread_rng();
    let adjectives = [
        "fast", "quick", "smart", "cool", "nice", "good", "best", "top", "new", "old",
    ];
    let nouns = [
        "host", "server", "node", "box", "machine", "system", "unit", "core", "hub", "gate",
    ];

    loop {
        let adj = adjectives[rng.gen_range(0..adjectives.len())];
        let noun = nouns[rng.gen_range(0..nouns.len())];
        let num = rng.gen_range(1..1000);

        let candidate = format!("{}-{}-{}", adj, noun, num);

        if !existing_names.contains(&candidate) {
            return candidate;
        }
    }
}
