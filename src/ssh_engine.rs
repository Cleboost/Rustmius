use ssh2::Session;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;
use std::io::Write;
use anyhow::Context;
use crate::config_observer::{SshHost, expand_tilde};

#[allow(dead_code)]
pub fn connect(host: &str, _user: &str) -> anyhow::Result<Session> {
    let addrs = format!("{}:22", host).to_socket_addrs()
        .with_context(|| format!("Failed to resolve {}:22", host))?;
    let mut tcp_opt = None;
    for addr in addrs {
        if let Ok(stream) = TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
            tcp_opt = Some(stream);
            break;
        }
    }
    let tcp = tcp_opt.ok_or_else(|| anyhow::anyhow!("Connection timeout to {}:22", host))?;
    let mut sess = Session::new()
        .context("Failed to create SSH session")?;
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .context("SSH handshake failed")?;

    Ok(sess)
}

pub fn deploy_pubkey(host: &SshHost, password: Option<String>, pubkey_content: &str) -> anyhow::Result<()> {
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
            println!("[DEBUG] SSH deploy connected to {} via Configure SSH Key ({})", host.hostname, key_path);
            authenticated = true;
        }
    }
    if !authenticated
        && sess.userauth_agent(user).is_ok() {
            println!("[DEBUG] SSH deploy connected to {} via SSH Agent", host.hostname);
            authenticated = true;
        }

    if !authenticated
        && let Some(pass) = password
            && sess.userauth_password(user, &pass).is_ok() {
                println!("[DEBUG] SSH deploy connected to {} via Password", host.hostname);
                authenticated = true;
            }
    if !authenticated {
        return Err(anyhow::anyhow!("Authentication failed (tried key, password, and agent)"));
    }
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
}