use ssh2::Session;
use std::time::Duration;
use std::io::{Read, Write};
use anyhow::Context;
use crate::config_observer::{SshHost, expand_tilde, REMOTE_SSH_DIR, REMOTE_AUTHORIZED_KEYS};
use tokio::net::{TcpStream, lookup_host};
use tracing::{info, debug, warn, instrument};

#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias))]
pub async fn establish_ssh_session(host: &SshHost, password: Option<&str>) -> anyhow::Result<Session> {
    let port = host.port.unwrap_or(22);
    let addr_str = format!("{}:{}", host.hostname, port);
    
    debug!("Resolving address {}", addr_str);
    let addrs = lookup_host(&addr_str).await
        .with_context(|| format!("Failed to resolve {}", addr_str))?;
    
    let mut tcp_opt = None;
    for addr in addrs {
        debug!("Attempting TCP connection to {}", addr);
        if let Ok(stream) = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
            if let Ok(stream) = stream {
                debug!("TCP connection established to {}", addr);
                tcp_opt = Some(stream);
                break;
            }
        }
    }
    
    let tcp = tcp_opt.ok_or_else(|| anyhow::anyhow!("Connection timeout to {}", host.hostname))?;
    let std_tcp = tcp.into_std()?;
    std_tcp.set_nonblocking(false)?;

    let host_cloned = host.clone();
    let password_cloned = password.map(|s| s.to_string());
    tokio::task::spawn_blocking(move || -> anyhow::Result<Session> {
        debug!("Starting SSH handshake");
        let mut sess = Session::new()
            .context("Failed to create SSH session object")?;
        sess.set_tcp_stream(std_tcp);
        sess.handshake()
            .context("SSH handshake failed - check network or firewall")?;

        let user = host_cloned.user.as_deref().unwrap_or("root");
        let mut authenticated = false;
        let mut auth_errors = Vec::new();

        debug!("Attempting authentication for user: {}", user);
        if let Some(ref key_path) = host_cloned.identity_file {
            debug!("Trying public key authentication: {}", key_path);
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
            debug!("Trying agent authentication");
            if sess.userauth_agent(user).is_ok() {
                info!("Authenticated via SSH agent");
                authenticated = true;
            } else {
                debug!("Agent auth failed or no agent running");
                auth_errors.push("Agent auth failed or no agent running".to_string());
            }
        }

        if !authenticated && let Some(ref pass) = password_cloned {
            debug!("Trying password authentication");
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
    }).await?
}

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
        debug!("Executing deployment command: {}", cmd);
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

#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, command = %command))]
pub async fn run_remote_command(host: &SshHost, password: Option<&str>, command: &str) -> anyhow::Result<String> {
    debug!("Running remote command on {}: {}", host.alias, command);
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
        debug!("Command output received ({} bytes)", output.len());
        Ok(output)
    }).await?
}