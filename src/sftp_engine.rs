use ssh2::Session;
use std::net::TcpStream;
use crate::config_observer::SshHost;
use std::path::Path;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

fn connect_sftp(host: SshHost, password: Option<String>) -> anyhow::Result<ssh2::Sftp> {
    let tcp = TcpStream::connect(format!("{}:22", host.hostname))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    let user = host.user.unwrap_or_else(|| "root".to_string());
    if let Some(pass) = password {
        sess.userauth_password(&user, &pass)?;
    } else {
        sess.userauth_agent(&user)?;
    }

    Ok(sess.sftp()?)
}

pub async fn list_files(host: SshHost, password: Option<String>, path: String) -> anyhow::Result<Vec<RemoteFile>> {
    tokio::task::spawn_blocking(move || {
        let sftp = connect_sftp(host, password)?;
        let dir = sftp.readdir(Path::new(&path))?;
        
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
        let sftp = connect_sftp(host, password)?;
        let p = Path::new(&path);
        if is_dir {
            sftp.rmdir(p)?;
        } else {
            sftp.unlink(p)?;
        }
        Ok(())
    }).await?
}

pub async fn create_dir(host: SshHost, password: Option<String>, path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let sftp = connect_sftp(host, password)?;
        sftp.mkdir(Path::new(&path), 0o755)?;
        Ok(())
    }).await?
}

pub async fn rename_file(host: SshHost, password: Option<String>, old_path: String, new_path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let sftp = connect_sftp(host, password)?;
        sftp.rename(Path::new(&old_path), Path::new(&new_path), None)?;
        Ok(())
    }).await?
}

pub async fn upload_file(host: SshHost, password: Option<String>, local_path: String, remote_path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let sftp = connect_sftp(host, password)?;
        let mut local_file = std::fs::File::open(local_path)?;
        let mut remote_file = sftp.create(Path::new(&remote_path))?;
        
        let mut buffer = [0; 16384];
        while let Ok(n) = local_file.read(&mut buffer) {
            if n == 0 { break; }
            remote_file.write_all(&buffer[..n])?;
        }
        Ok(())
    }).await?
}

pub async fn download_file(host: SshHost, password: Option<String>, remote_path: String, local_path: String) -> anyhow::Result<()> {
    tokio::task::spawn_blocking(move || {
        let sftp = connect_sftp(host, password)?;
        let mut remote_file = sftp.open(Path::new(&remote_path))?;
        let mut local_file = std::fs::File::create(local_path)?;
        
        let mut buffer = [0; 16384];
        while let Ok(n) = remote_file.read(&mut buffer) {
            if n == 0 { break; }
            local_file.write_all(&buffer[..n])?;
        }
        Ok(())
    }).await?
}
