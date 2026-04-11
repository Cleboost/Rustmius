use ssh2::Session;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use crate::config_observer::{SshHost, expand_tilde};
use std::path::Path;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

pub struct ActiveSession {
    _sess: Session,
    pub sftp: ssh2::Sftp,
}

fn get_session_pool() -> &'static Mutex<HashMap<String, Arc<ActiveSession>>> {
    static POOL: OnceLock<Mutex<HashMap<String, Arc<ActiveSession>>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn get_or_connect_sftp(host: &SshHost, password: &Option<String>) -> anyhow::Result<Arc<ActiveSession>> {
    let host_key = format!("{}@{}", host.user.as_deref().unwrap_or("root"), host.hostname);
    if let Ok(mut pool) = get_session_pool().lock() {
        if let Some(active) = pool.get(&host_key) {
            if active.sftp.stat(Path::new(".")).is_ok() {
                return Ok(active.clone());
            } else {
                pool.remove(&host_key);
            }
        }
    }

    let port = host.port.unwrap_or(22);
    let addrs = format!("{}:{}", host.hostname, port).to_socket_addrs()?;
    let mut tcp_opt = None;
    for addr in addrs {
        if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
            tcp_opt = Some(stream);
            break;
        }
    }
    let tcp = tcp_opt.ok_or_else(|| anyhow::anyhow!("Connection timeout to {}", host.hostname))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    let user = host.user.as_deref().unwrap_or("root");
    let mut authenticated = false;
    if let Some(ref key_path) = host.identity_file {
        let path = expand_tilde(key_path);
        if sess.userauth_pubkey_file(user, None, &path, None).is_ok() {
            println!("[DEBUG] SFTP connected to {} via Configure SSH Key ({})", host.hostname, key_path);
            authenticated = true;
        }
    }
    if !authenticated {
        if sess.userauth_agent(user).is_ok() {
            println!("[DEBUG] SFTP connected to {} via SSH Agent", host.hostname);
            authenticated = true;
        }
    }

    if !authenticated {
        if let Some(pass) = password {
            if sess.userauth_password(user, pass).is_ok() {
                println!("[DEBUG] SFTP connected to {} via Password", host.hostname);
                authenticated = true;
            }
        }
    }
    if !authenticated {
        return Err(anyhow::anyhow!("Authentication failed (tried key, password, and agent)"));
    }

    let sftp = sess.sftp()?;
    let active = Arc::new(ActiveSession { _sess: sess, sftp });
    if let Ok(mut pool) = get_session_pool().lock() {
        pool.insert(host_key, active.clone());
    }
    Ok(active)
}

pub async fn list_files(host: SshHost, password: Option<String>, path: String) -> anyhow::Result<Vec<RemoteFile>> {
    tokio::task::spawn_blocking(move || {
        let active = get_or_connect_sftp(&host, &password)?;
        let dir = active.sftp.readdir(Path::new(&path))?;
        let mut files = Vec::new();
        for (path, stat) in dir {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                files.push(RemoteFile {
                    name: name.to_string(),
                    is_dir: stat.is_dir(),
                    size: stat.size.unwrap_or(0),
                });
            }
        }
        files.sort_by(|a, b| {
            if a.is_dir != b.is_dir {
                b.is_dir.cmp(&a.is_dir)
            } else {
                a.name.to_lowercase().cmp(&b.name.to_lowercase())
            }
        });

        Ok(files)
    }).await?
}

pub async fn delete_file(host: SshHost, password: Option<String>, path: String, is_dir: bool) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let active = get_or_connect_sftp(&host, &password)?;
        let p = Path::new(&path);
        if is_dir {
            active.sftp.rmdir(p)?;
        } else {
            active.sftp.unlink(p)?;
        }
        Ok(())
    }).await?
}

pub async fn create_dir(host: SshHost, password: Option<String>, path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let active = get_or_connect_sftp(&host, &password)?;
        active.sftp.mkdir(Path::new(&path), 0o755)?;
        Ok(())
    }).await?
}

pub async fn create_file(host: SshHost, password: Option<String>, path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let active = get_or_connect_sftp(&host, &password)?;
        active.sftp.create(Path::new(&path))?;
        Ok(())
    }).await?
}

pub async fn rename_file(host: SshHost, password: Option<String>, old_path: String, new_path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let active = get_or_connect_sftp(&host, &password)?;
        active.sftp.rename(Path::new(&old_path), Path::new(&new_path), None)?;
        Ok(())
    }).await?
}

pub async fn upload_file(host: SshHost, password: Option<String>, local_path: String, remote_path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let active = get_or_connect_sftp(&host, &password)?;
        let mut local_file = std::fs::File::open(local_path)?;
        let mut remote_file = active.sftp.create(Path::new(&remote_path))?;
        let mut buffer = [0; 16384];
        while let Ok(n) = local_file.read(&mut buffer) {
            if n == 0 { break; }
            remote_file.write_all(&buffer[..n])?;
        }
        Ok(())
    }).await?
}

#[allow(dead_code)]
pub async fn download_file(host: SshHost, password: Option<String>, remote_path: String, local_path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        download_file_sync(host, password, remote_path, local_path)
    }).await?
}

pub fn download_file_sync(host: SshHost, password: Option<String>, remote_path: String, local_path: String) -> anyhow::Result<()> {
    let active = get_or_connect_sftp(&host, &password)?;
    let mut remote_file = active.sftp.open(Path::new(&remote_path))?;
    let mut local_file = std::fs::File::create(local_path)?;

    let mut buffer = [0; 16384];
    while let Ok(n) = remote_file.read(&mut buffer) {
        if n == 0 { break; }
        local_file.write_all(&buffer[..n])?;
    }
    Ok(())
}