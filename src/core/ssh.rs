use crate::storage::{AuthMethod, Host, OsKind};
use anyhow::{Context, Result};
use russh::client::{self, Handle};
use russh::keys::{PrivateKeyWithHashAlg, load_secret_key};
use russh::{ChannelMsg, Disconnect};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum SshCommand {
    Data(Vec<u8>),
    Resize { cols: u32, rows: u32 },
    Close,
}

#[derive(Debug)]
pub enum SshEvent {
    Connected,
    OsDetected(OsKind),
    Output(Vec<u8>),
    Error(String),
    Closed,
}

pub struct SshSession {
    pub cmd_tx: mpsc::UnboundedSender<SshCommand>,
    pub evt_rx: mpsc::UnboundedReceiver<SshEvent>,
}

struct ClientHandler;

impl client::Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &russh::keys::PublicKey,
    ) -> Result<bool, Self::Error> {
        // TODO: vérifier ~/.ssh/known_hosts et demander confirmation TOFU
        Ok(true)
    }
}

pub fn spawn_ssh_session(
    rt: &tokio::runtime::Handle,
    host: Host,
    cols: u32,
    rows: u32,
    ctx: egui::Context,
) -> SshSession {
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (evt_tx, evt_rx) = mpsc::unbounded_channel();

    rt.spawn(async move {
        if let Err(err) = run_ssh(host, cols, rows, cmd_rx, evt_tx.clone(), ctx.clone()).await {
            let _ = evt_tx.send(SshEvent::Error(format!("{err:#}")));
            ctx.request_repaint();
        }
        let _ = evt_tx.send(SshEvent::Closed);
        ctx.request_repaint();
    });

    SshSession { cmd_tx, evt_rx }
}

fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix("~/") {
        if let Some(dirs) = directories::UserDirs::new() {
            return dirs.home_dir().join(stripped);
        }
    }
    PathBuf::from(path)
}

async fn authenticate(session: &mut Handle<ClientHandler>, host: &Host) -> Result<bool> {
    match &host.auth {
        AuthMethod::Password => {
            let password = host.password.as_deref().unwrap_or("");
            let res = session
                .authenticate_password(&host.username, password)
                .await?;
            Ok(res.success())
        }
        AuthMethod::PrivateKey { path } => {
            let key_path = expand_tilde(path);
            let key_pair = load_secret_key(&key_path, None)
                .with_context(|| format!("Impossible de charger la clé privée {}", key_path.display()))?;
            let res = session
                .authenticate_publickey(
                    &host.username,
                    PrivateKeyWithHashAlg::new(
                        Arc::new(key_pair),
                        session.best_supported_rsa_hash().await?.flatten(),
                    ),
                )
                .await?;
            Ok(res.success())
        }
        AuthMethod::Agent => {
            // Tente les clés par défaut dans ~/.ssh/ si aucun agent n'est utilisé
            let candidates = ["~/.ssh/id_ed25519", "~/.ssh/id_rsa", "~/.ssh/id_ecdsa"];
            for candidate in candidates {
                let key_path = expand_tilde(candidate);
                if key_path.exists() {
                    if let Ok(key_pair) = load_secret_key(&key_path, None) {
                        let hash_alg = session.best_supported_rsa_hash().await?.flatten();
                        if let Ok(res) = session
                            .authenticate_publickey(
                                &host.username,
                                PrivateKeyWithHashAlg::new(Arc::new(key_pair), hash_alg),
                            )
                            .await
                        {
                            if res.success() {
                                return Ok(true);
                            }
                        }
                    }
                }
            }
            if let Some(pw) = &host.password {
                let res = session.authenticate_password(&host.username, pw).await?;
                return Ok(res.success());
            }
            Ok(false)
        }
    }
}

async fn run_ssh(
    host: Host,
    cols: u32,
    rows: u32,
    mut cmd_rx: mpsc::UnboundedReceiver<SshCommand>,
    evt_tx: mpsc::UnboundedSender<SshEvent>,
    ctx: egui::Context,
) -> Result<()> {
    let config = Arc::new(client::Config::default());
    let addr = format!("{}:{}", host.hostname, host.port);
    let mut session = client::connect(config, &addr, ClientHandler)
        .await
        .with_context(|| format!("Connexion TCP impossible vers {addr}"))?;

    if !authenticate(&mut session, &host).await? {
        anyhow::bail!("Échec d'authentification SSH pour {}@{addr}", host.username);
    }

    if let Some(os) = detect_remote_os(&mut session).await {
        let _ = evt_tx.send(SshEvent::OsDetected(os));
        ctx.request_repaint();
    }

    let mut channel = session.channel_open_session().await?;
    channel
        .request_pty(false, "xterm-256color", cols, rows, 0, 0, &[])
        .await?;
    channel.request_shell(false).await?;

    let _ = evt_tx.send(SshEvent::Connected);
    ctx.request_repaint();

    loop {
        tokio::select! {
            msg = channel.wait() => {
                match msg {
                    Some(ChannelMsg::Data { data }) => {
                        let _ = evt_tx.send(SshEvent::Output(data.to_vec()));
                        ctx.request_repaint();
                    }
                    Some(ChannelMsg::ExtendedData { data, .. }) => {
                        let _ = evt_tx.send(SshEvent::Output(data.to_vec()));
                        ctx.request_repaint();
                    }
                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) | None => {
                        break;
                    }
                    _ => {}
                }
            }
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(SshCommand::Data(bytes)) => {
                        channel.data(&bytes[..]).await?;
                    }
                    Some(SshCommand::Resize { cols, rows }) => {
                        channel.window_change(cols, rows, 0, 0).await?;
                    }
                    Some(SshCommand::Close) | None => {
                        let _ = channel.eof().await;
                        let _ = session.disconnect(Disconnect::ByApplication, "", "en").await;
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn detect_remote_os(session: &mut Handle<ClientHandler>) -> Option<OsKind> {
    let Ok(mut ch) = session.channel_open_session().await else {
        return None;
    };
    if ch
        .exec(true, "uname -s 2>/dev/null; cat /etc/os-release 2>/dev/null")
        .await
        .is_err()
    {
        return None;
    }

    let mut out = Vec::new();
    let _ = tokio::time::timeout(std::time::Duration::from_secs(2), async {
        while let Some(msg) = ch.wait().await {
            match msg {
                ChannelMsg::Data { data } => out.extend_from_slice(&data),
                ChannelMsg::Eof | ChannelMsg::Close => break,
                _ => {}
            }
        }
    })
    .await;

    let text = String::from_utf8_lossy(&out);
    let os = parse_os_release(&text);
    if os == OsKind::Unknown {
        None
    } else {
        Some(os)
    }
}

pub fn parse_os_release(output: &str) -> OsKind {
    let lower = output.to_ascii_lowercase();
    for line in lower.lines() {
        let trimmed = line.trim();
        if let Some(val) = trimmed.strip_prefix("id=") {
            let id = val.trim_matches('"').trim_matches('\'');
            match id {
                "debian" => return OsKind::Debian,
                "ubuntu" => return OsKind::Ubuntu,
                "arch" | "archarm" => return OsKind::Arch,
                "fedora" => return OsKind::Fedora,
                "nixos" => return OsKind::NixOS,
                "alpine" => return OsKind::Alpine,
                "centos" => return OsKind::CentOs,
                "rhel" | "redhat" => return OsKind::RedHat,
                "rocky" => return OsKind::Rocky,
                "almalinux" => return OsKind::Alma,
                "kali" => return OsKind::Kali,
                "linuxmint" => return OsKind::Mint,
                "opensuse" | "opensuse-tumbleweed" | "opensuse-leap" | "sles" => {
                    return OsKind::OpenSuse;
                }
                "gentoo" => return OsKind::Gentoo,
                "manjaro" => return OsKind::Manjaro,
                _ => {}
            }
        }
    }

    if lower.contains("darwin") {
        OsKind::MacOs
    } else if lower.contains("freebsd") {
        OsKind::FreeBsd
    } else if lower.contains("windows") || lower.contains("mingw") {
        OsKind::Windows
    } else if lower.contains("linux") {
        OsKind::Linux
    } else {
        OsKind::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_debian_and_nixos_from_os_release() {
        let deb = "Linux\nPRETTY_NAME=\"Debian GNU/Linux 12 (bookworm)\"\nID=debian\n";
        assert_eq!(parse_os_release(deb), OsKind::Debian);

        let nix = "Linux\nID=nixos\nNAME=NixOS\n";
        assert_eq!(parse_os_release(nix), OsKind::NixOS);

        assert_eq!(parse_os_release("Darwin\n"), OsKind::MacOs);
        assert_eq!(parse_os_release("Linux\n"), OsKind::Linux);
    }
}
