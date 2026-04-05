use ssh2::Session;
use std::net::TcpStream;
use anyhow::Context;

pub fn connect(host: &str, _user: &str) -> anyhow::Result<Session> {
    let tcp = TcpStream::connect(format!("{}:22", host))
        .with_context(|| format!("Failed to connect to {}:22", host))?;
    
    let mut sess = Session::new()
        .context("Failed to create SSH session")?;
    
    sess.set_tcp_stream(tcp);
    sess.handshake()
        .context("SSH handshake failed")?;

    // Note: Authentication will be handled in a later task (MVP 0.8)
    // but the engine is ready for it.
    
    Ok(sess)
}
