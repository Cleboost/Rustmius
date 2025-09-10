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

        // Ignorer les lignes vides et les commentaires
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Nouveau Host
        if line.to_lowercase().starts_with("host ") {
            // Sauvegarder le serveur précédent s'il existe
            if let Some(server) = current_server.take() {
                servers.push(server);
            }

            // Créer un nouveau serveur
            let host_name = line[5..].trim().to_string();
            current_server = Some(SshServer {
                name: host_name,
                hostname: None,
                user: None,
                identity_file: None,
                port: None,
            });
        } else if let Some(ref mut server) = current_server {
            // Parser les directives du serveur actuel
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
                    _ => {} // Ignorer les autres directives
                }
            }
        }
    }

    // Ajouter le dernier serveur s'il existe
    if let Some(server) = current_server {
        servers.push(server);
    }

    Ok(servers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh_config() {
        let config_content = r#"
Host condat-basket
  HostName 82.165.91.168
  User root
  IdentityFile ~/.ssh/cleboost

Host snipy
  HostName 217.154.7.198
  User root
  IdentityFile ~/.ssh/cleboost

Host aur.archlinux.org
  IdentityFile ~/.ssh/aur
  User aur
"#;

        let servers = parse_ssh_config(config_content).unwrap();

        assert_eq!(servers.len(), 3);

        // Vérifier le premier serveur
        assert_eq!(servers[0].name, "condat-basket");
        assert_eq!(servers[0].hostname, Some("82.165.91.168".to_string()));
        assert_eq!(servers[0].user, Some("root".to_string()));
        assert_eq!(
            servers[0].identity_file,
            Some("~/.ssh/cleboost".to_string())
        );

        // Vérifier le deuxième serveur
        assert_eq!(servers[1].name, "snipy");
        assert_eq!(servers[1].hostname, Some("217.154.7.198".to_string()));
        assert_eq!(servers[1].user, Some("root".to_string()));
        assert_eq!(
            servers[1].identity_file,
            Some("~/.ssh/cleboost".to_string())
        );

        // Vérifier le troisième serveur
        assert_eq!(servers[2].name, "aur.archlinux.org");
        assert_eq!(servers[2].hostname, None);
        assert_eq!(servers[2].user, Some("aur".to_string()));
        assert_eq!(servers[2].identity_file, Some("~/.ssh/aur".to_string()));
    }
}
