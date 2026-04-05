use ssh2::Session;
use std::net::TcpStream;
use crate::config_observer::SshHost;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

pub async fn list_files(host: SshHost, password: Option<String>, path: String) -> anyhow::Result<Vec<RemoteFile>> {
    tokio::task::spawn_blocking(move || {
        let tcp = TcpStream::connect(format!("{}:22", host.hostname))?;
        let mut sess = Session::new()?;
        sess.set_tcp_stream(tcp);
        sess.handshake()?;

        let user = host.user.unwrap_or_else(|| "root".to_string());
        if let Some(pass) = password {
            sess.userauth_password(&user, &pass)?;
        } else {
            // Try agent if no password
            sess.userauth_agent(&user)?;
        }

        let sftp = sess.sftp()?;
        let mut dir = sftp.readdir(Path::new(&path))?;
        
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
        
        // Sort: directories first, then alphabetical
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
