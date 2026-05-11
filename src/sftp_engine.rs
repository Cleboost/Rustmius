use ssh2::Session;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use anyhow::Context;
use crate::config_observer::SshHost;
use tracing::{info, debug, warn, instrument};

/// Represents a file or directory on a remote server.
#[derive(Debug, Clone)]
pub struct RemoteFile {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
}

/// Manages an active SSH session and its associated SFTP subsystem.
pub struct ActiveSession {
    _sess: Session,
    pub sftp: ssh2::Sftp,
}

type HostLock = tokio::sync::Mutex<Option<Arc<ActiveSession>>>;

fn get_session_pool() -> &'static Mutex<HashMap<String, Arc<HostLock>>> {
    static POOL: OnceLock<Mutex<HashMap<String, Arc<HostLock>>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias))]
async fn get_or_connect_sftp(host: &SshHost, password: Option<&str>) -> anyhow::Result<Arc<ActiveSession>> {
    let host_key = format!("{}@{}", host.user.as_deref().unwrap_or("root"), host.hostname);
    
    let host_lock = {
        let mut pool = get_session_pool().lock().map_err(|_| anyhow::anyhow!("Pool lock poisoned"))?;
        pool.entry(host_key).or_insert_with(|| Arc::new(tokio::sync::Mutex::new(None))).clone()
    };

    let mut session_guard = host_lock.lock().await;
    
    if let Some(active) = &*session_guard {
        debug!("Checking existing SFTP session");
        if active.sftp.stat(Path::new(".")).is_ok() {
            debug!("Using existing SFTP session");
            return Ok(active.clone());
        }
        debug!("Existing SFTP session stale, reconnecting");
    }

    info!("Connecting new SFTP session to {}", host.alias);
    let sess = crate::ssh_engine::establish_ssh_session(host, password).await?;

    let sftp_sess = sess.clone();
    let sftp = tokio::task::spawn_blocking(move || -> anyhow::Result<ssh2::Sftp> {
        debug!("Initializing SFTP subsystem");
        sftp_sess.sftp()
            .context("Failed to initialize SFTP subsystem")
    }).await??;

    let active = Arc::new(ActiveSession { _sess: sess, sftp });
    *session_guard = Some(active.clone());
    
    info!("SFTP session established for {}", host.alias);
    Ok(active)
}

/// Lists files in the specified remote directory.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, path = %path))]
pub async fn list_files(host: &SshHost, password: Option<&str>, path: &str) -> anyhow::Result<Vec<RemoteFile>> {
    debug!("Listing files in {}", path);
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<Vec<RemoteFile>> {
        let dir = active.sftp.readdir(Path::new(&path_owned))
            .with_context(|| format!("Failed to read directory: {}", path_owned))?;
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

/// Deletes a file or directory on the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, path = %path, is_dir = is_dir))]
pub async fn delete_file(host: &SshHost, password: Option<&str>, path: &str, is_dir: bool) -> anyhow::Result<()> {
    info!("Deleting {} (dir: {})", path, is_dir);
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let p = Path::new(&path_owned);
        if is_dir {
            active.sftp.rmdir(p)
                .with_context(|| format!("Failed to delete directory: {}", path_owned))?;
        } else {
            active.sftp.unlink(p)
                .with_context(|| format!("Failed to delete file: {}", path_owned))?;
        }
        Ok(())
    }).await?
}

/// Creates a new directory on the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, path = %path))]
pub async fn create_dir(host: &SshHost, password: Option<&str>, path: &str) -> anyhow::Result<()> {
    info!("Creating directory {}", path);
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        active.sftp.mkdir(Path::new(&path_owned), 0o755)
            .with_context(|| format!("Failed to create directory: {}", path_owned))?;
        Ok(())
    }).await?
}

/// Creates an empty file on the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, path = %path))]
pub async fn create_file(host: &SshHost, password: Option<&str>, path: &str) -> anyhow::Result<()> {
    info!("Creating file {}", path);
    let active = get_or_connect_sftp(host, password).await?;
    let path_owned = path.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        active.sftp.create(Path::new(&path_owned))
            .with_context(|| format!("Failed to create file: {}", path_owned))?;
        Ok(())
    }).await?
}

/// Renames or moves a file or directory on the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, old_path = %old_path, new_path = %new_path))]
pub async fn rename_file(host: &SshHost, password: Option<&str>, old_path: &str, new_path: &str) -> anyhow::Result<()> {
    info!("Renaming {} to {}", old_path, new_path);
    let active = get_or_connect_sftp(host, password).await?;
    let old_owned = old_path.to_string();
    let new_owned = new_path.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        active.sftp.rename(Path::new(&old_owned), Path::new(&new_owned), None)
            .with_context(|| format!("Failed to rename {} to {}", old_owned, new_owned))?;
        Ok(())
    }).await?
}

/// Uploads a local file to the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, local = %local_path, remote = %remote_path))]
pub async fn upload_file(host: &SshHost, password: Option<&str>, local_path: &str, remote_path: &str) -> anyhow::Result<()> {
    info!("Uploading {} to {}", local_path, remote_path);
    let active = get_or_connect_sftp(host, password).await?;
    let local_owned = local_path.to_string();
    let remote_owned = remote_path.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut local_file = std::fs::File::open(&local_owned)
            .with_context(|| format!("Failed to open local file for upload: {}", local_owned))?;
        let mut remote_file = active.sftp.create(Path::new(&remote_owned))
            .with_context(|| format!("Failed to create remote file: {}", remote_owned))?;
        std::io::copy(&mut local_file, &mut remote_file)
            .context("Failed to copy data during upload")?;
        Ok(())
    }).await?
}

/// Downloads a remote file to the local filesystem.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, remote = %remote_path, local = %local_path))]
pub async fn download_file(host: &SshHost, password: Option<&str>, remote_path: &str, local_path: &str) -> anyhow::Result<()> {
    info!("Downloading {} to {}", remote_path, local_path);
    let active = get_or_connect_sftp(host, password).await?;
    let remote_owned = remote_path.to_string();
    let local_owned = local_path.to_string();
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut remote_file = active.sftp.open(Path::new(&remote_owned))
            .with_context(|| format!("Failed to open remote file: {}", remote_owned))?;
        let mut local_file = std::fs::File::create(&local_owned)
            .with_context(|| format!("Failed to create local file: {}", local_owned))?;
        std::io::copy(&mut remote_file, &mut local_file)
            .context("Failed to copy data during download")?;
        Ok(())
    }).await?
}

/// Synchronously downloads a remote file using a provided Tokio runtime handle.
#[instrument(skip(rt, password), fields(host = %host.hostname, alias = %host.alias, remote = %remote_path, local = %local_path))]
pub fn download_file_sync(rt: tokio::runtime::Handle, host: SshHost, password: Option<String>, remote_path: String, local_path: String) -> anyhow::Result<()> {
    info!("Sync downloading {} to {}", remote_path, local_path);
    let active = rt.block_on(get_or_connect_sftp(&host, password.as_deref()))
        .context("Failed to connect for sync download")?;
    
    let mut remote_file = active.sftp.open(Path::new(&remote_path))
        .with_context(|| format!("Failed to open remote file: {}", remote_path))?;
    let mut local_file = std::fs::File::create(&local_path)
        .with_context(|| format!("Failed to create local file: {}", local_path))?;
    std::io::copy(&mut remote_file, &mut local_file)
        .context("Failed to copy data during sync download")?;
    Ok(())
}