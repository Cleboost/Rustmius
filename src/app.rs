use crate::core::ssh::{SshCommand, SshEvent, SshSession, spawn_ssh_session};
use crate::core::terminal::TerminalState;
use crate::storage::ssh_config::{find_unimported_ssh_hosts, import_ssh_config_into_vault};
use crate::storage::store::{default_vault_path, load_plain_vault, save_plain_vault};
use crate::storage::{Host, HostSource, OsKind, Vault};
use crate::ui::background::paint_backdrop;
use crate::ui::connecting::{ConnectingAction, render_connecting_screen};
use crate::ui::host_grid::{HostGridAction, HostSortMode, HostViewMode, render_host_grid};
use crate::ui::host_modal::{HostModalAction, HostModalState, render_host_panel};
use crate::ui::sidebar::{NavSection, render_sidebar};
use crate::ui::terminal_view::render_terminal;
use crate::ui::theme;
use crate::ui::titlebar::{ActiveTab, SessionTabView, TitlebarAction, render_titlebar};
use egui::{Align2, CursorIcon, ResizeDirection, ViewportCommand};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use uuid::Uuid;

struct TerminalTab {
    id: Uuid,
    host_id: Uuid,
    os: OsKind,
    source: HostSource,
    title: String,
    hostname: String,
    port: u16,
    connected: bool,
    show_logs: bool,
    logs: Vec<String>,
    terminal: TerminalState,
    ssh: SshSession,
}

pub struct RustmiusApp {
    rt: Runtime,
    vault_path: PathBuf,
    vault: Vault,
    ghost_hosts: Vec<Host>,
    nav: NavSection,
    active: ActiveTab,
    quick_connect: String,
    empty_host_input: String,
    tabs: Vec<TerminalTab>,
    modal: Option<HostModalState>,
    panel_open: bool,
    view_mode: HostViewMode,
    view_menu_open: bool,
    sort_mode: HostSortMode,
    sort_menu_open: bool,
}

impl RustmiusApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_termius_theme(&cc.egui_ctx);
        let rt = Runtime::new().expect("Impossible de démarrer le runtime Tokio");
        let vault_path = default_vault_path().expect("Impossible de résoudre le chemin du coffre");
        let vault = load_plain_vault(&vault_path).unwrap_or_else(|e| {
            tracing::error!("Erreur chargement vault.json: {e:#}");
            Vault::new()
        });
        if !vault_path.exists() {
            Self::persist_vault(&vault_path, &vault);
        }
        let ghost_hosts = find_unimported_ssh_hosts(&vault);

        Self {
            rt,
            vault_path,
            vault,
            ghost_hosts,
            nav: NavSection::Hosts,
            active: ActiveTab::Vaults,
            quick_connect: String::new(),
            empty_host_input: String::new(),
            tabs: Vec::new(),
            modal: None,
            panel_open: false,
            view_mode: HostViewMode::Grid,
            view_menu_open: false,
            sort_mode: HostSortMode::NewestFirst,
            sort_menu_open: false,
        }
    }

    fn persist_vault(path: &PathBuf, vault: &Vault) {
        if let Err(e) = save_plain_vault(path, vault) {
            tracing::error!("Erreur sauvegarde coffre: {e:#}");
        }
    }

    fn connect_host(&mut self, host: Host, ctx: &egui::Context) {
        let id = Uuid::new_v4();
        let ssh = spawn_ssh_session(self.rt.handle(), host.clone(), 80, 24, ctx.clone());
        let initial_log = format!("Connecting to {}@{}:{}...", host.username, host.hostname, host.port);
        self.tabs.push(TerminalTab {
            id,
            host_id: host.id,
            os: host.os,
            source: host.source,
            title: host.label,
            hostname: host.hostname,
            port: host.port,
            connected: false,
            show_logs: false,
            logs: vec![initial_log],
            terminal: TerminalState::new(80, 24),
            ssh,
        });
        self.active = ActiveTab::Session(id);
    }

    fn close_session(&mut self, id: Uuid) {
        if let Some(pos) = self.tabs.iter().position(|t| t.id == id) {
            let removed = self.tabs.remove(pos);
            let _ = removed.ssh.cmd_tx.send(SshCommand::Close);
            if self.active == ActiveTab::Session(id) {
                self.active = self
                    .tabs
                    .last()
                    .map(|t| ActiveTab::Session(t.id))
                    .unwrap_or(ActiveTab::Vaults);
            }
        }
    }

    fn drain_ssh_events(&mut self) {
        let mut os_updates = Vec::new();
        for tab in self.tabs.iter_mut() {
            while let Ok(evt) = tab.ssh.evt_rx.try_recv() {
                match evt {
                    SshEvent::Connected => {
                        tab.connected = true;
                        tab.logs.push("SSH session established.".to_string());
                    }
                    SshEvent::OsDetected(os) => {
                        tab.os = os;
                        tab.logs.push(format!("Detected OS: {os:?}"));
                        os_updates.push((tab.host_id, os));
                    }
                    SshEvent::Output(bytes) => {
                        tab.connected = true;
                        tab.terminal.feed_bytes(&bytes);
                    }
                    SshEvent::Error(err) => {
                        tab.connected = true;
                        tab.logs.push(format!("Error: {err}"));
                        tab.terminal
                            .feed_bytes(format!("\r\n\x1b[31m[Erreur SSH] {err}\x1b[0m\r\n").as_bytes());
                    }
                    SshEvent::Closed => {
                        tab.connected = true;
                        tab.logs.push("Session closed.".to_string());
                        tab.terminal
                            .feed_bytes(b"\r\n\x1b[33m[Session ferm\xc3\xa9e]\x1b[0m\r\n");
                    }
                }
            }
        }
        if !os_updates.is_empty() {
            let mut changed = false;
            for (host_id, os) in os_updates {
                if let Some(h) = self.vault.hosts.iter_mut().find(|h| h.id == host_id) {
                    if h.os != os {
                        h.os = os;
                        changed = true;
                    }
                }
            }
            if changed {
                Self::persist_vault(&self.vault_path, &self.vault);
            }
        }
    }

    fn handle_host_action(&mut self, action: HostGridAction, ctx: &egui::Context) {
        match action {
            HostGridAction::Connect(host) => self.connect_host(host, ctx),
            HostGridAction::QuickConnect(cmd) => {
                if let Some(host) = parse_quick_connect(&cmd) {
                    self.quick_connect.clear();
                    self.connect_host(host, ctx);
                }
            }
            HostGridAction::NewHost => {
                self.modal = Some(HostModalState::new_empty());
                self.panel_open = true;
            }
            HostGridAction::NewHostWithAddress(addr) => {
                let mut m = HostModalState::new_empty();
                m.hostname = addr.clone();
                m.label = addr;
                self.modal = Some(m);
                self.panel_open = true;
            }
            HostGridAction::EditHost(host) => {
                self.modal = Some(HostModalState::from_host(&host));
                self.panel_open = true;
            }
            HostGridAction::DeleteHost(id) => {
                self.vault.hosts.retain(|h| h.id != id);
                Self::persist_vault(&self.vault_path, &self.vault);
                self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
            }
            HostGridAction::ImportSshConfig => {
                if import_ssh_config_into_vault(&mut self.vault) > 0 {
                    Self::persist_vault(&self.vault_path, &self.vault);
                    self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
                }
            }
            HostGridAction::ImportGhost(host) => {
                self.vault.hosts.push(host);
                Self::persist_vault(&self.vault_path, &self.vault);
                self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
            }
            HostGridAction::None => {}
        }
    }
}

fn parse_quick_connect(input: &str) -> Option<Host> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    let mut target = None;
    let mut port: u16 = 22;
    let mut i = 0;
    while i < parts.len() {
        if parts[i] == "ssh" {
            i += 1;
            continue;
        }
        if parts[i] == "-p" && i + 1 < parts.len() {
            if let Ok(p) = parts[i + 1].parse::<u16>() {
                port = p;
            }
            i += 2;
            continue;
        }
        if !parts[i].starts_with('-') && target.is_none() {
            target = Some(parts[i]);
        }
        i += 1;
    }

    let target = target?;
    let (user, hostname) = match target.split_once('@') {
        Some((u, h)) => (u.to_string(), h.to_string()),
        None => (
            std::env::var("USER").unwrap_or_else(|_| "root".to_string()),
            target.to_string(),
        ),
    };
    let mut host = Host::new(hostname.clone(), hostname, user);
    host.port = port;
    Some(host)
}

/// Edge-resize support for the frameless window.
fn handle_window_resize(ctx: &egui::Context) {
    if ctx.input(|i| i.viewport().maximized.unwrap_or(false)) {
        return;
    }
    let rect = ctx.screen_rect();
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else {
        return;
    };
    const M: f32 = 5.0;
    let (l, r) = (pos.x <= rect.left() + M, pos.x >= rect.right() - M);
    let (t, b) = (pos.y <= rect.top() + M, pos.y >= rect.bottom() - M);
    let dir = match (l, r, t, b) {
        (true, _, true, _) => Some((ResizeDirection::NorthWest, CursorIcon::ResizeNorthWest)),
        (_, true, true, _) => Some((ResizeDirection::NorthEast, CursorIcon::ResizeNorthEast)),
        (true, _, _, true) => Some((ResizeDirection::SouthWest, CursorIcon::ResizeSouthWest)),
        (_, true, _, true) => Some((ResizeDirection::SouthEast, CursorIcon::ResizeSouthEast)),
        (true, ..) => Some((ResizeDirection::West, CursorIcon::ResizeWest)),
        (_, true, ..) => Some((ResizeDirection::East, CursorIcon::ResizeEast)),
        (_, _, true, _) => Some((ResizeDirection::North, CursorIcon::ResizeNorth)),
        (_, _, _, true) => Some((ResizeDirection::South, CursorIcon::ResizeSouth)),
        _ => None,
    };
    if let Some((d, cursor)) = dir {
        ctx.set_cursor_icon(cursor);
        if ctx.input(|i| i.pointer.primary_pressed()) {
            ctx.send_viewport_cmd(ViewportCommand::BeginResize(d));
        }
    }
}

fn placeholder(ui: &mut egui::Ui, text: &str) {
    let rect = ui.max_rect();
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        text,
        theme::medium(14.0),
        theme::TEXT_MUTED,
    );
}

impl eframe::App for RustmiusApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        theme::BG_BASE.to_normalized_gamma_f32()
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.drain_ssh_events();
        paint_backdrop(ctx, ctx.screen_rect());

        // ---- Title bar
        let sessions: Vec<(Uuid, OsKind, String)> =
            self.tabs.iter().map(|t| (t.id, t.os, t.title.clone())).collect();
        let mut tb_action = TitlebarAction::None;
        egui::TopBottomPanel::top("titlebar")
            .exact_height(theme::TITLEBAR_H)
            .show_separator_line(false)
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let views: Vec<SessionTabView> = sessions
                    .iter()
                    .map(|(id, os, t)| SessionTabView {
                        id: *id,
                        os: *os,
                        title: t,
                    })
                    .collect();
                tb_action = render_titlebar(ui, self.active, &views);
            });
        match tb_action {
            TitlebarAction::Select(tab) => self.active = tab,
            TitlebarAction::CloseSession(id) => self.close_session(id),
            TitlebarAction::None => {}
        }

        // ---- Sidebar (Vaults view only)
        if self.active == ActiveTab::Vaults {
            egui::SidePanel::left("sidebar")
                .exact_width(theme::SIDEBAR_W)
                .resizable(false)
                .show_separator_line(false)
                .frame(egui::Frame::NONE)
                .show(ctx, |ui| render_sidebar(ui, &mut self.nav));
        }

        // ---- Host editor right panel (smooth slide-in/out)
        let anim_t = ctx.animate_bool_with_time(egui::Id::new("host_panel_anim"), self.panel_open, 0.18);
        if anim_t > 0.0 && self.active == ActiveTab::Vaults {
            if let Some(m) = &mut self.modal {
                let mut panel_action = HostModalAction::None;
                let width = (324.0 * anim_t).max(1.0);
                egui::SidePanel::right("host_editor_panel")
                    .exact_width(width)
                    .resizable(false)
                    .show_separator_line(false)
                    .frame(egui::Frame::NONE)
                    .show(ctx, |ui| {
                        panel_action = render_host_panel(ui, m);
                    });

                match panel_action {
                    HostModalAction::Save(host) => {
                        if let Some(existing) = self.vault.hosts.iter_mut().find(|h| h.id == host.id) {
                            *existing = host;
                        } else {
                            self.vault.hosts.push(host);
                        }
                        Self::persist_vault(&self.vault_path, &self.vault);
                        self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
                        self.panel_open = false;
                    }
                    HostModalAction::SaveAndConnect(host) => {
                        if let Some(existing) = self.vault.hosts.iter_mut().find(|h| h.id == host.id) {
                            *existing = host.clone();
                        } else {
                            self.vault.hosts.push(host.clone());
                        }
                        Self::persist_vault(&self.vault_path, &self.vault);
                        self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
                        self.panel_open = false;
                        self.connect_host(host, ctx);
                    }
                    HostModalAction::Cancel => {
                        self.panel_open = false;
                    }
                    HostModalAction::None => {}
                }
            }
        } else if !self.panel_open && anim_t == 0.0 {
            self.modal = None;
        }

        // ---- Main content
        let mut host_action = HostGridAction::None;
        let mut close_tab_id: Option<Uuid> = None;
        egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| match self.active {
            ActiveTab::Session(id) => {
                if let Some(tab) = self.tabs.iter_mut().find(|t| t.id == id) {
                    if tab.connected {
                        render_terminal(ui, &mut tab.terminal, &tab.ssh.cmd_tx);
                    } else {
                        match render_connecting_screen(
                            ui,
                            &tab.title,
                            &tab.hostname,
                            tab.port,
                            tab.os,
                            &tab.source,
                            tab.show_logs,
                            &tab.logs,
                        ) {
                            ConnectingAction::Close => close_tab_id = Some(tab.id),
                            ConnectingAction::ToggleLogs => tab.show_logs = !tab.show_logs,
                            ConnectingAction::None => {}
                        }
                    }
                }
            }
            ActiveTab::Sftp => placeholder(ui, "SFTP — coming soon"),
            ActiveTab::Vaults => match self.nav {
                NavSection::Hosts => {
                    host_action = render_host_grid(
                        ui,
                        &self.vault,
                        &self.ghost_hosts,
                        &mut self.quick_connect,
                        &mut self.empty_host_input,
                        &mut self.view_mode,
                        &mut self.view_menu_open,
                        &mut self.sort_mode,
                        &mut self.sort_menu_open,
                    );
                }
                NavSection::Keychain => placeholder(ui, "Keychain — coming soon"),
                NavSection::PortForwarding => placeholder(ui, "Port Forwarding — coming soon"),
                NavSection::Snippets => placeholder(ui, "Snippets — coming soon"),
                NavSection::KnownHosts => placeholder(ui, "Known Hosts — coming soon"),
                NavSection::Logs => placeholder(ui, "Logs — coming soon"),
            },
        });
        self.handle_host_action(host_action, ctx);
        if let Some(cid) = close_tab_id {
            self.close_session(cid);
        }

        handle_window_resize(ctx);
    }
}
