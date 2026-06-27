use ssh2::Session;
use std::time::Duration;
use std::io::{Read, Write};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use anyhow::Context;
use crate::config_observer::{SshHost, expand_tilde, REMOTE_SSH_DIR, REMOTE_AUTHORIZED_KEYS};
use tokio::net::{TcpStream, lookup_host};
use tracing::{info, warn, instrument};

type SharedSession = Arc<tokio::sync::Mutex<Option<Session>>>;

/// Restores the previous `SSH_AUTH_SOCK` value when dropped, so temporarily
/// pointing libssh2 at a host-specific `IdentityAgent` socket doesn't leak.
struct SshAuthSockGuard {
    previous: Option<std::ffi::OsString>,
}

impl Drop for SshAuthSockGuard {
    fn drop(&mut self) {
        // SAFETY: runs on the same blocking thread that set the variable.
        unsafe {
            match self.previous.take() {
                Some(value) => std::env::set_var("SSH_AUTH_SOCK", value),
                None => std::env::remove_var("SSH_AUTH_SOCK"),
            }
        }
    }
}

fn get_ssh_pool() -> &'static Mutex<HashMap<String, SharedSession>> {
    static POOL: OnceLock<Mutex<HashMap<String, SharedSession>>> = OnceLock::new();
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Establishes an SSH session with the given host, attempting multiple authentication methods.
/// Reuses an existing session if available and healthy.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias))]
pub async fn establish_ssh_session(host: &SshHost, password: Option<&str>) -> anyhow::Result<Session> {
    let host_key = format!("{}@{}", host.user.as_deref().unwrap_or("root"), host.hostname);
    
    let shared_session = {
        let mut pool = get_ssh_pool().lock().map_err(|_| anyhow::anyhow!("SSH Pool lock poisoned"))?;
        pool.entry(host_key).or_insert_with(|| Arc::new(tokio::sync::Mutex::new(None))).clone()
    };

    let mut guard = shared_session.lock().await;
    
    if let Some(sess) = &*guard {
        tracing::trace!("Checking existing SSH session health");
        if sess.authenticated() {
            // Quick check: try to open a channel
            let sess_clone = sess.clone();
            let is_alive = tokio::task::spawn_blocking(move || {
                sess_clone.channel_session().is_ok()
            }).await.unwrap_or(false);

            if is_alive {
                tracing::trace!("Reusing existing SSH session");
                return Ok(sess.clone());
            }
        }
        tracing::trace!("Existing SSH session stale, reconnecting");
    }

    let port = host.port.unwrap_or(22);
    let addr_str = format!("{}:{}", host.hostname, port);
    
    tracing::trace!("Resolving address {}", addr_str);
    let addrs = lookup_host(&addr_str).await
        .with_context(|| format!("Failed to resolve {}", addr_str))?;
    
    let mut tcp_opt = None;
    for addr in addrs {
        tracing::trace!("Attempting TCP connection to {}", addr);
        if let Ok(stream) = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
            if let Ok(stream) = stream {
                tracing::trace!("TCP connection established to {}", addr);
                tcp_opt = Some(stream);
                break;
            }
        }
    }
    
    let tcp = tcp_opt.ok_or_else(|| anyhow::anyhow!("Connection timeout to {}", host.hostname))?;
    let std_tcp = tcp.into_std()?;
    std_tcp.set_nonblocking(false)?;

    let host_cloned = host.clone();
    // Keep the in-flight password copy in a buffer that is zeroed on drop.
    let password_cloned = password.map(|s| zeroize::Zeroizing::new(s.to_string()));
    let sess = tokio::task::spawn_blocking(move || -> anyhow::Result<Session> {
        tracing::trace!("Starting SSH handshake");
        let mut sess = Session::new()
            .context("Failed to create SSH session object")?;
        sess.set_tcp_stream(std_tcp);
        sess.handshake()
            .context("SSH handshake failed - check network or firewall")?;

        let user = host_cloned.user.as_deref().unwrap_or("root");
        let mut authenticated = false;
        let mut auth_errors = Vec::new();

        tracing::trace!("Attempting authentication for user: {}", user);
        if let Some(ref key_path) = host_cloned.identity_file {
            tracing::trace!("Trying public key authentication: {}", key_path);
            let path = expand_tilde(key_path);
            match sess.userauth_pubkey_file(user, None, &path, None) {
                Ok(_) => {
                    info!("Authenticated via public key: {}", key_path);
                    authenticated = true;
                },
                Err(e) => {
                    warn!("Key auth failed: {}", e);
                    auth_errors.push(format!("Key auth failed: {}", e));
                },
            }
        }

        if !authenticated {
            // libssh2 (via the ssh2 crate) only discovers the agent socket
            // through `SSH_AUTH_SOCK`. When the host declares an `IdentityAgent`
            // in the SSH config (e.g. 1Password's `~/.1password/agent.sock`),
            // point the agent at that socket so it behaves like the system
            // `ssh` client used by the terminal.
            let _agent_sock_guard = host_cloned.identity_agent.as_deref().map(|agent_path| {
                let expanded = expand_tilde(agent_path);
                let previous = std::env::var_os("SSH_AUTH_SOCK");
                tracing::trace!("Using IdentityAgent socket: {}", expanded.display());
                // SAFETY: SSH auth runs serially within this blocking task and
                // the previous value is restored when the guard drops.
                unsafe { std::env::set_var("SSH_AUTH_SOCK", &expanded); }
                SshAuthSockGuard { previous }
            });

            tracing::trace!("Trying agent authentication");
            if sess.userauth_agent(user).is_ok() {
                info!("Authenticated via SSH agent");
                authenticated = true;
            } else {
                tracing::trace!("Agent auth failed or no agent running");
                auth_errors.push("Agent auth failed or no agent running".to_string());
            }
        }

        if !authenticated && let Some(ref pass) = password_cloned {
            tracing::trace!("Trying password authentication");
            match sess.userauth_password(user, pass) {
                Ok(_) => {
                    info!("Authenticated via password");
                    authenticated = true;
                },
                Err(e) => {
                    warn!("Password auth failed: {}", e);
                    auth_errors.push(format!("Password auth failed: {}", e));
                },
            }
        }

        if authenticated {
            Ok(sess)
        } else {
            let combined = auth_errors.join("; ");
            warn!("Authentication failed for {}@{}: {}", user, host_cloned.hostname, combined);
            Err(anyhow::anyhow!("Authentication failed for {}@{}. Details: {}", user, host_cloned.hostname, combined))
        }
    }).await??;

    *guard = Some(sess.clone());
    Ok(sess)
}

/// Deploys a public key to the remote host's `authorized_keys` file.
#[instrument(skip(password, pubkey_content), fields(host = %host.hostname, alias = %host.alias))]
pub async fn deploy_pubkey(host: &SshHost, password: Option<&str>, pubkey_content: &str) -> anyhow::Result<()> {
    info!("Deploying public key to {}", host.alias);
    let sess = establish_ssh_session(host, password).await?;
    let pubkey_owned = pubkey_content.to_string();
    
    tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut channel = sess.channel_session()
            .context("Failed to open SSH channel for deployment")?;
        let cmd = format!(
            "mkdir -p {0} && chmod 700 {0} && cat >> {1} && chmod 600 {1}",
            REMOTE_SSH_DIR, REMOTE_AUTHORIZED_KEYS
        );
        tracing::trace!("Executing deployment command: {}", cmd);
        channel.exec(&cmd)
            .context("Failed to execute pubkey deployment command")?;

        if pubkey_owned.ends_with('\n') {
            channel.write_all(pubkey_owned.as_bytes())
                .context("Failed to write pubkey to authorized_keys")?;
        } else {
            let mut content = pubkey_owned;
            content.push('\n');
            channel.write_all(content.as_bytes())
                .context("Failed to write pubkey to authorized_keys")?;
        }
        channel.send_eof()?;
        channel.wait_eof()?;
        channel.close()?;
        channel.wait_close()?;
        Ok(())
    }).await??;
    info!("Public key deployed successfully to {}", host.alias);
    Ok(())
}

/// Runs a command on the remote host and returns its standard output.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, command = %command))]
pub async fn run_remote_command(host: &SshHost, password: Option<&str>, command: &str) -> anyhow::Result<String> {
    tracing::trace!("Running remote command on {}: {}", host.alias, command);
    let sess = establish_ssh_session(host, password).await
        .context("Failed to establish SSH session for remote command")?;
    let cmd_owned = command.to_string();

    tokio::task::spawn_blocking(move || -> anyhow::Result<String> {
        let mut channel = sess.channel_session()
            .context("Failed to open SSH channel for command execution")?;
        channel.exec(&cmd_owned)
            .with_context(|| format!("Failed to execute command: {}", cmd_owned))?;
        let mut output = String::new();
        channel.read_to_string(&mut output)
            .context("Failed to read command output")?;
        channel.wait_close()?;
        tracing::trace!("Command output received ({} bytes)", output.len());
        Ok(output)
    }).await?
}