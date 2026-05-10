use ssh2::Session;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use crate::config_observer::SshHost;

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

type HostLock = tokio::sync::Mutex<Option<Arc<ActiveSession>>>;

fn get_session_pool() -> &'static Mutex<HashMap<String, Arc<HostLock>>> {
    static POOL: OnceLock<Mutex<HashMap<String, Arc<HostLock>>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

async fn get_or_connect_sftp(host: &SshHost, password: Option<&str>) -> anyhow::Result<Arc<ActiveSession>> {
    let host_key = format!("{}@{}", host.user.as_deref().unwrap_or("root"), host.hostname);
    
    let host_lock = {
        let mut pool = get_session_pool().lock().map_err(|_| anyhow::anyhow!("Pool lock poisoned"))?;
        pool.entry(host_key).or_insert_with(|| Arc::new(tokio::sync::Mutex::new(None))).clone()
    };

    let mut session_guard = host_lock.lock().await;
    
    if let Some(active) = &*session_guard {
        if active.sftp.stat(Path::new(".")).is_ok() {
            return Ok(active.clone());
        }
    }

    let sess = crate::ssh_engine::establish_ssh_session(host, password).await?;

    let sftp_sess = sess.clone();
    let sftp = tokio::task::spawn_blocking(move || {
        sftp_sess.sftp()
    }).await??;

    let active = Arc::new(ActiveSession { _sess: sess, sftp });
    *session_guard = Some(active.clone());
    
    Ok(active)
}

pub async fn list_files(host: &SshHost, password: Option<&str>, path: &str) -> anyhow::Result<Vec<RemoteFile>> {
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || {
        let dir = active.sftp.readdir(Path::new(&path_owned))?;
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

pub async fn delete_file(host: &SshHost, password: Option<&str>, path: &str, is_dir: bool) -> anyhow::Result<()> {
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || {
        let p = Path::new(&path_owned);
        if is_dir {
            active.sftp.rmdir(p)?;
        } else {
            active.sftp.unlink(p)?;
        }
        Ok(())
    }).await?
}

pub async fn create_dir(host: &SshHost, password: Option<&str>, path: &str) -> anyhow::Result<()> {
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || {
        active.sftp.mkdir(Path::new(&path_owned), 0o755)?;
        Ok(())
    }).await?
}

pub async fn create_file(host: &SshHost, password: Option<&str>, path: &str) -> anyhow::Result<()> {
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || {
        active.sftp.create(Path::new(&path_owned))?;
        Ok(())
    }).await?
}

pub async fn rename_file(host: &SshHost, password: Option<&str>, old_path: &str, new_path: &str) -> anyhow::Result<()> {
    let active = get_or_connect_sftp(host, password).await?;
    let old_owned = old_path.to_string();
    let new_owned = new_path.to_string();
    tokio::task::spawn_blocking(move || {
        active.sftp.rename(Path::new(&old_owned), Path::new(&new_owned), None)?;
        Ok(())
    }).await?
}

pub async fn upload_file(host: &SshHost, password: Option<&str>, local_path: &str, remote_path: &str) -> anyhow::Result<()> {
    let active = get_or_connect_sftp(host, password).await?;
    let local_owned = local_path.to_string();
    let remote_owned = remote_path.to_string();
    tokio::task::spawn_blocking(move || {
        let mut local_file = std::fs::File::open(local_owned)?;
        let mut remote_file = active.sftp.create(Path::new(&remote_owned))?;
        std::io::copy(&mut local_file, &mut remote_file)?;
        Ok(())
    }).await?
}

#[allow(dead_code)]
pub async fn download_file(host: &SshHost, password: Option<&str>, remote_path: &str, local_path: &str) -> anyhow::Result<()> {
    let active = get_or_connect_sftp(host, password).await?;
    let remote_owned = remote_path.to_string();
    let local_owned = local_path.to_string();
    tokio::task::spawn_blocking(move || {
        let mut remote_file = active.sftp.open(Path::new(&remote_owned))?;
        let mut local_file = std::fs::File::create(local_owned)?;
        std::io::copy(&mut remote_file, &mut local_file)?;
        Ok(())
    }).await?
}

pub fn download_file_sync(host: &SshHost, password: Option<&str>, remote_path: &str, local_path: &str) -> anyhow::Result<()> {
    let runtime = tokio::runtime::Handle::current();
    let active = runtime.block_on(get_or_connect_sftp(host, password))?;
    
    let mut remote_file = active.sftp.open(Path::new(remote_path))?;
    let mut local_file = std::fs::File::create(local_path)?;
    std::io::copy(&mut remote_file, &mut local_file)?;
    Ok(())
}