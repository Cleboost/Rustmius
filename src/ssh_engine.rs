use ssh2::Session;
use std::net::TcpStream;
use std::io::Write;
use anyhow::Context;
use crate::config_observer::SshHost;

#[allow(dead_code)]
pub fn connect(host: &str, _user: &str) -> anyhow::Result<Session> {
    let tcp = TcpStream::connect(format!("{}:22", host))
        .with_context(|| format!("Failed to connect to {}:22", host))?;
    
    let mut sess = Session::new()
        .context("Failed to create SSH session")?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .context("SSH handshake failed")?;

    Ok(sess)
}

pub fn deploy_pubkey(host: &SshHost, password: Option<String>, pubkey_content: &str) -> anyhow::Result<()> {
    let port = host.port.unwrap_or(22);
    let tcp = TcpStream::connect(format!("{}:{}", host.hostname, port))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    
    let user = host.user.as_deref().unwrap_or("root");
    let mut authenticated = false;
    
    if let Some(ref key_path) = host.identity_file {
        let path = std::path::Path::new(key_path);
        if sess.userauth_pubkey_file(user, None, path, None).is_ok() {
            println!("[DEBUG] SSH deploy connected to {} via Configure SSH Key ({})", host.hostname, key_path);
            authenticated = true;
        }
    }
    
    if !authenticated {
        if let Some(pass) = password {
            if sess.userauth_password(user, &pass).is_ok() {
                println!("[DEBUG] SSH deploy connected to {} via Password", host.hostname);
                authenticated = true;
            }
        }
    }
    
    if !authenticated {
        if sess.userauth_agent(user).is_ok() {
            println!("[DEBUG] SSH deploy connected to {} via SSH Agent", host.hostname);
            authenticated = true;
        }
    }
    
    if !authenticated {
        return Err(anyhow::anyhow!("Authentication failed (tried key, password, and agent)"));
    }
    
    let mut channel = sess.channel_session()?;
    channel.exec("mkdir -p ~/.ssh && chmod 700 ~/.ssh && cat >> ~/.ssh/authorized_keys && chmod 600 ~/.ssh/authorized_keys")?;
    
    channel.write_all(pubkey_content.as_bytes())?;
    channel.send_eof()?;
    channel.wait_eof()?;
    channel.close()?;
    channel.wait_close()?;
    
    Ok(())
}
