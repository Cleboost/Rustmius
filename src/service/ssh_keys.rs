use base64::{Engine as _, engine::general_purpose};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SshKey {
    pub name: String,
    pub key_type: String,
    pub fingerprint: String,
    pub file_path: String,
    pub has_public: bool,
    pub has_private: bool,
}

impl SshKey {
    pub fn new(
        name: String,
        key_type: String,
        fingerprint: String,
        file_path: String,
        has_public: bool,
        has_private: bool,
    ) -> Self {
        Self {
            name,
            key_type,
            fingerprint,
            file_path,
            has_public,
            has_private,
        }
    }
}

pub fn load_ssh_keys() -> Result<Vec<SshKey>, Box<dyn std::error::Error>> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Impossible de trouver le répertoire home")?
        .join(".ssh");

    if !ssh_dir.exists() {
        return Ok(vec![]);
    }

    let mut keys = Vec::new();
    let mut key_pairs: HashMap<String, (Option<String>, Option<String>)> = HashMap::new();

    let entries = fs::read_dir(&ssh_dir)?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Nom de fichier invalide")?;

        if file_name == "config" || file_name.starts_with("known_hosts") {
            continue;
        }

        let is_public = file_name.ends_with(".pub");
        let base_name = if is_public {
            file_name.strip_suffix(".pub").unwrap_or(file_name)
        } else {
            file_name
        };

        let entry = key_pairs
            .entry(base_name.to_string())
            .or_insert((None, None));
        if is_public {
            entry.0 = Some(file_name.to_string());
        } else {
            entry.1 = Some(file_name.to_string());
        }
    }

    for (base_name, (pub_file, priv_file)) in key_pairs {
        let has_public = pub_file.is_some();
        let has_private = priv_file.is_some();

        if has_public || has_private {
            let key_file = pub_file.or(priv_file).unwrap();
            let key_path = ssh_dir.join(&key_file);

            if let Ok(key_info) = get_key_info(&key_path) {
                keys.push(SshKey::new(
                    base_name,
                    key_info.key_type,
                    key_info.fingerprint,
                    key_path.to_string_lossy().to_string(),
                    has_public,
                    has_private,
                ));
            }
        }
    }

    keys.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(keys)
}

pub fn regenerate_public_key(private_key_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let private_path = Path::new(private_key_path);
    let public_path = private_path.with_extension("pub");

    if !private_path.exists() {
        return Err(format!("La clé privée n'existe pas: {}", private_key_path).into());
    }

    let output = Command::new("ssh-keygen")
        .args(&["-y", "-f", private_key_path])
        .output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "Erreur lors de la génération de la clé publique: {}",
            error_msg
        )
        .into());
    }

    let public_key_content = String::from_utf8(output.stdout)?;
    fs::write(&public_path, public_key_content)?;

    Ok(public_path.to_string_lossy().to_string())
}

pub fn delete_key_pair(key_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Impossible de trouver le répertoire home")?
        .join(".ssh");

    let private_key_path = ssh_dir.join(key_name);
    let public_key_path = ssh_dir.join(format!("{}.pub", key_name));

    let mut deleted_files = Vec::new();

    if private_key_path.exists() {
        fs::remove_file(&private_key_path)?;
        deleted_files.push(format!("Clé privée: {}", private_key_path.display()));
    }

    if public_key_path.exists() {
        fs::remove_file(&public_key_path)?;
        deleted_files.push(format!("Clé publique: {}", public_key_path.display()));
    }

    if deleted_files.is_empty() {
        return Err(format!("Aucune clé trouvée pour '{}'", key_name).into());
    }

    println!("Clés supprimées avec succès:");
    for file in deleted_files {
        println!("  - {}", file);
    }

    Ok(())
}

pub fn read_key_content(
    key_name: &str,
) -> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Impossible de trouver le répertoire home")?
        .join(".ssh");

    let private_key_path = ssh_dir.join(key_name);
    let public_key_path = ssh_dir.join(format!("{}.pub", key_name));

    let mut private_content = None;
    let mut public_content = None;

    if private_key_path.exists() {
        match fs::read_to_string(&private_key_path) {
            Ok(content) => private_content = Some(content),
            Err(e) => eprintln!("Erreur lors de la lecture de la clé privée: {}", e),
        }
    }

    if public_key_path.exists() {
        match fs::read_to_string(&public_key_path) {
            Ok(content) => public_content = Some(content),
            Err(e) => eprintln!("Erreur lors de la lecture de la clé publique: {}", e),
        }
    }

    if private_content.is_none() && public_content.is_none() {
        return Err(format!("Aucune clé trouvée pour '{}'", key_name).into());
    }

    Ok((private_content, public_content))
}

#[derive(Debug)]
struct KeyInfo {
    key_type: String,
    fingerprint: String,
}

fn get_key_info(key_path: &Path) -> Result<KeyInfo, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(key_path)?;
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Err("Fichier de clé vide".into());
    }

    let key_type = if content.contains("-----BEGIN RSA PRIVATE KEY-----") {
        "RSA".to_string()
    } else if content.contains("-----BEGIN OPENSSH PRIVATE KEY-----") {
        if content.contains("ssh-rsa") {
            "RSA".to_string()
        } else if content.contains("ssh-ed25519") {
            "Ed25519".to_string()
        } else if content.contains("ecdsa-sha2-nistp256") {
            "ECDSA P-256".to_string()
        } else if content.contains("ecdsa-sha2-nistp384") {
            "ECDSA P-384".to_string()
        } else if content.contains("ecdsa-sha2-nistp521") {
            "ECDSA P-521".to_string()
        } else {
            "OpenSSH".to_string()
        }
    } else if content.contains("-----BEGIN EC PRIVATE KEY-----") {
        "ECDSA".to_string()
    } else {
        let mut key_line = None;
        for line in &lines {
            let trimmed = line.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && !trimmed.starts_with("-----BEGIN")
                && !trimmed.starts_with("-----END")
            {
                key_line = Some(trimmed);
                break;
            }
        }

        let key_line = key_line.ok_or("Aucune ligne de clé valide trouvée")?;
        let parts: Vec<&str> = key_line.split_whitespace().collect();

        if parts.len() < 2 {
            return Err("Format de clé invalide".into());
        }

        match parts[0] {
            "ssh-rsa" => {
                let key_data = parts[1];
                match general_purpose::STANDARD.decode(key_data) {
                    Ok(decoded) => {
                        let key_size = decoded.len() * 8;
                        format!("RSA {}", key_size)
                    }
                    Err(_) => "RSA".to_string(),
                }
            }
            "ssh-ed25519" => "Ed25519".to_string(),
            "ecdsa-sha2-nistp256" => "ECDSA P-256".to_string(),
            "ecdsa-sha2-nistp384" => "ECDSA P-384".to_string(),
            "ecdsa-sha2-nistp521" => "ECDSA P-521".to_string(),
            other => other.to_string(),
        }
    };

    let fingerprint = format!(
        "SHA256:{}...",
        &key_path.file_name().unwrap_or_default().to_string_lossy()[..std::cmp::min(
            20,
            key_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .len()
        )]
    );

    Ok(KeyInfo {
        key_type,
        fingerprint,
    })
}

pub fn generate_ssh_key(
    name: &str,
    key_type: &str,
    key_length: u32,
    email: &str,
    passphrase: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let ssh_dir = dirs::home_dir()
        .ok_or("Impossible de trouver le répertoire home")?
        .join(".ssh");

    if !ssh_dir.exists() {
        fs::create_dir_all(&ssh_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&ssh_dir)?.permissions();
            perms.set_mode(0o700);
            fs::set_permissions(&ssh_dir, perms)?;
        }
    }

    let private_key_path = ssh_dir.join(name);
    let public_key_path = ssh_dir.join(format!("{}.pub", name));

    if private_key_path.exists() || public_key_path.exists() {
        return Err(format!("A key with name '{}' already exists", name).into());
    }

    let mut cmd = Command::new("ssh-keygen");
    cmd.arg("-t").arg(key_type.to_lowercase());

    if key_type == "RSA" || key_type == "ECDSA" {
        cmd.arg("-b").arg(&key_length.to_string());
    }

    cmd.arg("-f").arg(&private_key_path);

    if !email.is_empty() {
        cmd.arg("-C").arg(email);
    }

    if !passphrase.is_empty() {
        cmd.arg("-N").arg(passphrase);
    } else {
        cmd.arg("-N").arg("");
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let error_msg = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Erreur lors de la génération de la clé SSH: {}", error_msg).into());
    }

    if !private_key_path.exists() {
        return Err("La clé privée n'a pas été créée".into());
    }

    if !public_key_path.exists() {
        return Err("La clé publique n'a pas été créée".into());
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&private_key_path)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&private_key_path, perms)?;

        let mut perms = fs::metadata(&public_key_path)?.permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&public_key_path, perms)?;
    }

    println!("Clé SSH générée avec succès:");
    println!("  - Clé privée: {}", private_key_path.display());
    println!("  - Clé publique: {}", public_key_path.display());

    Ok(private_key_path.to_string_lossy().to_string())
}
