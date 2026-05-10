use ssh2::Session;
use std::time::Duration;
use std::io::{Read, Write};
use anyhow::Context;
use crate::config_observer::{SshHost, expand_tilde};
use tokio::net::{TcpStream, lookup_host};

pub async fn establish_ssh_session(host: &SshHost, password: Option<String>) -> anyhow::Result<Session> {
    let port = host.port.unwrap_or(22);
    let addr_str = format!("{}:{}", host.hostname, port);
    
    let addrs = lookup_host(&addr_str).await
        .with_context(|| format!("Failed to resolve {}", addr_str))?;
    
    let mut tcp_opt = None;
    for addr in addrs {
        if let Ok(stream) = tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(&addr)).await {
            if let Ok(stream) = stream {
                tcp_opt = Some(stream);
                break;
            }
        }
    }
    
    let tcp = tcp_opt.ok_or_else(|| anyhow::anyhow!("Connection timeout to {}", host.hostname))?;
    let std_tcp = tcp.into_std()?;
    std_tcp.set_nonblocking(false)?;

    let host_cloned = host.clone();
    tokio::task::spawn_blocking(move || {
        let mut sess = Session::new()
            .context("Failed to create SSH session")?;
        sess.set_tcp_stream(std_tcp);
        sess.handshake()
            .context("SSH handshake failed")?;

        let user = host_cloned.user.as_deref().unwrap_or("root");
        let mut authenticated = false;

        if let Some(ref key_path) = host_cloned.identity_file {
            let path = expand_tilde(key_path);
            if sess.userauth_pubkey_file(user, None, &path, None).is_ok() {
                authenticated = true;
            }
        }

        if !authenticated && sess.userauth_agent(user).is_ok() {
            authenticated = true;
        }

        if !authenticated && let Some(pass) = password {
            if sess.userauth_password(user, &pass).is_ok() {
                authenticated = true;
            }
        }

        if authenticated {
            Ok(sess)
        } else {
            Err(anyhow::anyhow!("Authentication failed for {}@{} (tried key, agent, and password)", user, host_cloned.hostname))
        }
    }).await?
}

pub async fn deploy_pubkey(host: SshHost, password: Option<String>, pubkey_content: String) -> anyhow::Result<()> {
    let sess = establish_ssh_session(&host, password).await?;
    
    tokio::task::spawn_blocking(move || {
        let mut channel = sess.channel_session()?;
        channel.exec("mkdir -p ~/.ssh && chmod 700 ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys")?;

        if pubkey_content.ends_with('\n') {
            channel.write_all(pubkey_content.as_bytes())?;
        } else {
            let mut content = pubkey_content.to_owned();
            content.push('\n');
            channel.write_all(content.as_bytes())?;
        }
        channel.send_eof()?;
        channel.wait_eof()?;
        channel.close()?;
        channel.wait_close()?;
        Ok(())
    }).await?
}

pub async fn run_remote_command(host: SshHost, password: Option<String>, command: String) -> anyhow::Result<String> {
    let sess = establish_ssh_session(&host, password).await?;

    tokio::task::spawn_blocking(move || {
        let mut channel = sess.channel_session()?;
        channel.exec(&command)?;
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output)
    }).await?
}