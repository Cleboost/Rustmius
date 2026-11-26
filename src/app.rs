use crate::core::ssh::{SshCommand, SshEvent, SshSession, spawn_ssh_session};
use crate::core::terminal::TerminalState;
use crate::i18n::tr;
use crate::storage::gist_sync::{
    SyncConfig, create_vault_gist, decode_encrypted_vault, encode_encrypted_vault,
    fetch_vault_gist, find_existing_vault_gist, load_sync_config, merge_vaults, save_sync_config,
    update_vault_gist,
};
use crate::storage::ssh_config::{find_unimported_ssh_hosts, import_ssh_config_into_vault};
use crate::storage::store::{
    AppSettings, default_settings_path, default_sync_config_path, default_vault_path,
    load_app_settings, load_plain_vault, save_app_settings, save_plain_vault,
};
use crate::storage::{Host, HostSource, OsKind, Vault};
use crate::ui::background::paint_backdrop;
use crate::ui::connecting::{ConnectingAction, render_connecting_screen};
use crate::ui::host_grid::{HostGridAction, HostSortMode, HostViewMode, render_host_grid};
use crate::ui::host_modal::{HostModalAction, HostModalState, render_host_panel};
use crate::ui::settings_modal::{
    SettingsModalState, SyncOnboardingStep, SyncUiAction, render_settings_modal,
};
use crate::ui::sidebar::{NavSection, render_sidebar};
use crate::ui::terminal_view::render_terminal;
use crate::ui::theme;
use crate::ui::titlebar::{ActiveTab, SessionTabView, TitlebarAction, render_titlebar};
use chrono::Utc;
use egui::{Align2, Color32, CornerRadius, CursorIcon, ResizeDirection, RichText, Stroke, ViewportCommand, vec2};
use std::path::PathBuf;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uuid::Uuid;

enum SyncEvent {
    Created {
        cfg: SyncConfig,
        password: String,
    },
    Connected {
        cfg: SyncConfig,
        password: String,
        remote_vault: Vault,
    },
    Pulled {
        remote_vault: Vault,
    },
    Pushed,
    Error(String),
}

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
    sync_path: PathBuf,
    settings_path: PathBuf,
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
    new_host_menu_open: bool,
    titlebar_menu_open: bool,
    settings_modal: SettingsModalState,
    app_settings: AppSettings,
    sync_locked: bool,
    unlock_password: String,
    unlock_show_password: bool,
    unlock_error: Option<String>,
    sync_cfg: Option<SyncConfig>,
    master_password: Option<String>,
    sync_tx: mpsc::UnboundedSender<SyncEvent>,
    sync_rx: mpsc::UnboundedReceiver<SyncEvent>,
}

impl RustmiusApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        theme::apply_termius_theme(&cc.egui_ctx);
        let rt = Runtime::new().expect("Failed to start Tokio runtime");
        let vault_path = default_vault_path().expect("Failed to resolve vault path");
        let sync_path =
            default_sync_config_path().expect("Failed to resolve sync.enc path");
        let settings_path =
            default_settings_path().expect("Failed to resolve settings.json path");
        let app_settings = load_app_settings(&settings_path);
        let mut sync_locked = sync_path.exists();
        let mut sync_cfg = None;
        let mut master_password = None;
        let mut initial_pull = false;
        if sync_locked
            && let Some(pw) = &app_settings.auto_unlock_password
                && let Ok(cfg) = load_sync_config(&sync_path, pw) {
                    initial_pull = cfg.auto_sync;
                    sync_cfg = Some(cfg);
                    master_password = Some(pw.clone());
                    sync_locked = false;
                }

        let vault = load_plain_vault(&vault_path).unwrap_or_else(|e| {
            tracing::error!("Failed to load vault.json: {e:#}");
            Vault::new()
        });
        if !vault_path.exists() {
            let _ = save_plain_vault(&vault_path, &vault);
        }
        let ghost_hosts = find_unimported_ssh_hosts(&vault);
        let (sync_tx, sync_rx) = mpsc::unbounded_channel();

        let mut app = Self {
            rt,
            vault_path,
            sync_path,
            settings_path,
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
            new_host_menu_open: false,
            titlebar_menu_open: false,
            settings_modal: SettingsModalState::default(),
            app_settings,
            sync_locked,
            unlock_password: String::new(),
            unlock_show_password: false,
            unlock_error: None,
            sync_cfg,
            master_password,
            sync_tx,
            sync_rx,
        };
        if initial_pull {
            app.spawn_pull(cc.egui_ctx.clone());
        }
        app
    }

    fn persist_vault(&mut self, ctx: Option<&egui::Context>) {
        if let Err(e) = save_plain_vault(&self.vault_path, &self.vault) {
            tracing::error!("Failed to save vault: {e:#}");
        }
        if let (Some(cfg), Some(pw), Some(ctx)) = (&self.sync_cfg, &self.master_password, ctx)
            && cfg.auto_sync {
                self.spawn_push(cfg.github_token.clone(), cfg.gist_id.clone(), pw.clone(), ctx.clone());
            }
    }

    fn spawn_create_sync(&mut self, token: String, password: String, ctx: egui::Context) {
        let s = tr(self.app_settings.language);
        self.settings_modal.sync_busy = true;
        self.settings_modal.sync_error = None;
        self.settings_modal.sync_status = Some(s.sync_status_creating.to_string());
        let vault = self.vault.clone();
        let tx = self.sync_tx.clone();
        self.rt.spawn(async move {
            let res = async {
                let b64 = encode_encrypted_vault(&vault, &password)?;
                let gist_id = create_vault_gist(&token, &b64).await?;
                Ok::<SyncConfig, anyhow::Error>(SyncConfig {
                    github_token: token,
                    gist_id,
                    auto_sync: true,
                    last_sync: Some(Utc::now()),
                })
            }
            .await;
            match res {
                Ok(cfg) => {
                    let _ = tx.send(SyncEvent::Created { cfg, password });
                }
                Err(e) => {
                    let _ = tx.send(SyncEvent::Error(format!("{e:#}")));
                }
            }
            ctx.request_repaint();
        });
    }

    fn spawn_connect_existing(&mut self, token: String, password: String, ctx: egui::Context) {
        let s = tr(self.app_settings.language);
        let not_found_msg = s.sync_err_not_found;
        self.settings_modal.sync_busy = true;
        self.settings_modal.sync_error = None;
        self.settings_modal.sync_status = Some(s.sync_status_searching.to_string());
        let tx = self.sync_tx.clone();
        self.rt.spawn(async move {
            let res = async {
                let found = find_existing_vault_gist(&token).await?;
                let Some((gist_id, content)) = found else {
                    anyhow::bail!("{not_found_msg}");
                };
                let remote_vault = decode_encrypted_vault(&content, &password)?;
                let cfg = SyncConfig {
                    github_token: token,
                    gist_id,
                    auto_sync: true,
                    last_sync: Some(Utc::now()),
                };
                Ok::<(SyncConfig, Vault), anyhow::Error>((cfg, remote_vault))
            }
            .await;
            match res {
                Ok((cfg, remote_vault)) => {
                    let _ = tx.send(SyncEvent::Connected {
                        cfg,
                        password,
                        remote_vault,
                    });
                }
                Err(e) => {
                    let _ = tx.send(SyncEvent::Error(format!("{e:#}")));
                }
            }
            ctx.request_repaint();
        });
    }

    fn spawn_pull(&mut self, ctx: egui::Context) {
        let (Some(cfg), Some(pw)) = (&self.sync_cfg, &self.master_password) else {
            return;
        };
        let s = tr(self.app_settings.language);
        self.settings_modal.sync_busy = true;
        self.settings_modal.sync_error = None;
        self.settings_modal.sync_status = Some(s.sync_status_syncing.to_string());
        let token = cfg.github_token.clone();
        let gist_id = cfg.gist_id.clone();
        let password = pw.clone();
        let tx = self.sync_tx.clone();
        self.rt.spawn(async move {
            let res = async {
                let content = fetch_vault_gist(&token, &gist_id).await?;
                decode_encrypted_vault(&content, &password)
            }
            .await;
            match res {
                Ok(remote_vault) => {
                    let _ = tx.send(SyncEvent::Pulled { remote_vault });
                }
                Err(e) => {
                    let _ = tx.send(SyncEvent::Error(format!("{e:#}")));
                }
            }
            ctx.request_repaint();
        });
    }

    fn spawn_push(&self, token: String, gist_id: String, password: String, ctx: egui::Context) {
        let vault = self.vault.clone();
        let tx = self.sync_tx.clone();
        self.rt.spawn(async move {
            let res = async {
                let b64 = encode_encrypted_vault(&vault, &password)?;
                update_vault_gist(&token, &gist_id, &b64).await
            }
            .await;
            match res {
                Ok(()) => {
                    let _ = tx.send(SyncEvent::Pushed);
                }
                Err(e) => {
                    let _ = tx.send(SyncEvent::Error(format!("{e:#}")));
                }
            }
            ctx.request_repaint();
        });
    }

    fn drain_sync_events(&mut self, ctx: &egui::Context) {
        let s = tr(self.app_settings.language);
        while let Ok(evt) = self.sync_rx.try_recv() {
            match evt {
                SyncEvent::Created { cfg, password } => {
                    let _ = save_sync_config(&self.sync_path, &cfg, &password);
                    self.sync_cfg = Some(cfg);
                    self.master_password = Some(password);
                    self.settings_modal.sync_busy = false;
                    self.settings_modal.sync_error = None;
                    self.settings_modal.sync_status = Some(s.sync_status_created.to_string());
                    self.settings_modal.token_input.clear();
                    self.settings_modal.password_input.clear();
                    self.settings_modal.confirm_password_input.clear();
                }
                SyncEvent::Connected {
                    cfg,
                    password,
                    remote_vault,
                } => {
                    self.vault = merge_vaults(&self.vault, &remote_vault);
                    self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
                    let _ = save_plain_vault(&self.vault_path, &self.vault);
                    let _ = save_sync_config(&self.sync_path, &cfg, &password);
                    let token = cfg.github_token.clone();
                    let gist_id = cfg.gist_id.clone();
                    self.sync_cfg = Some(cfg);
                    self.master_password = Some(password.clone());
                    self.settings_modal.sync_busy = false;
                    self.settings_modal.sync_error = None;
                    self.settings_modal.sync_status = Some(s.sync_status_connected.to_string());
                    self.settings_modal.token_input.clear();
                    self.settings_modal.password_input.clear();
                    self.settings_modal.confirm_password_input.clear();
                    self.spawn_push(token, gist_id, password, ctx.clone());
                }
                SyncEvent::Pulled { remote_vault } => {
                    self.vault = merge_vaults(&self.vault, &remote_vault);
                    self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
                    let _ = save_plain_vault(&self.vault_path, &self.vault);
                    if let (Some(cfg), Some(pw)) = (&mut self.sync_cfg, &self.master_password) {
                        cfg.last_sync = Some(Utc::now());
                        let _ = save_sync_config(&self.sync_path, cfg, pw);
                        let token = cfg.github_token.clone();
                        let gist_id = cfg.gist_id.clone();
                        let password = pw.clone();
                        self.spawn_push(token, gist_id, password, ctx.clone());
                    }
                    self.settings_modal.sync_busy = false;
                    self.settings_modal.sync_status = Some(s.sync_status_synced.to_string());
                }
                SyncEvent::Pushed => {
                    if let (Some(cfg), Some(pw)) = (&mut self.sync_cfg, &self.master_password) {
                        cfg.last_sync = Some(Utc::now());
                        let _ = save_sync_config(&self.sync_path, cfg, pw);
                    }
                    self.settings_modal.sync_busy = false;
                    self.settings_modal.sync_status = Some(s.sync_status_synced.to_string());
                }
                SyncEvent::Error(err) => {
                    self.settings_modal.sync_busy = false;
                    self.settings_modal.sync_status = None;
                    self.settings_modal.sync_error = Some(err);
                }
            }
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

    fn drain_ssh_events(&mut self, ctx: &egui::Context) {
        let s = tr(self.app_settings.language);
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
                        tab.terminal.feed_bytes(
                            format!("\r\n\x1b[31m[{}] {err}\x1b[0m\r\n", s.term_ssh_error)
                                .as_bytes(),
                        );
                    }
                    SshEvent::Closed => {
                        tab.connected = true;
                        tab.logs.push("Session closed.".to_string());
                        tab.terminal.feed_bytes(
                            format!("\r\n\x1b[33m[{}]\x1b[0m\r\n", s.term_session_closed)
                                .as_bytes(),
                        );
                    }
                }
            }
        }
        if !os_updates.is_empty() {
            let mut changed = false;
            for (host_id, os) in os_updates {
                if let Some(h) = self.vault.hosts.iter_mut().find(|h| h.id == host_id)
                    && h.os != os {
                        h.os = os;
                        h.updated_at = Utc::now();
                        changed = true;
                    }
            }
            if changed {
                self.persist_vault(Some(ctx));
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
                self.vault.deleted_hosts.insert(id, Utc::now());
                self.persist_vault(Some(ctx));
                self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
            }
            HostGridAction::ImportSshConfig => {
                if import_ssh_config_into_vault(&mut self.vault) > 0 {
                    self.persist_vault(Some(ctx));
                    self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
                }
            }
            HostGridAction::ImportGhost(host) => {
                self.vault.hosts.push(host);
                self.persist_vault(Some(ctx));
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
        self.drain_ssh_events(ctx);
        self.drain_sync_events(ctx);
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
                tb_action = render_titlebar(
                    ui,
                    self.active,
                    &views,
                    &mut self.titlebar_menu_open,
                    self.app_settings.language,
                );
            });

        let s = tr(self.app_settings.language);
        if self.sync_locked {
            self.titlebar_menu_open = false;
            egui::CentralPanel::default().frame(egui::Frame::NONE).show(ctx, |ui| {
                let area = ui.max_rect();
                let card_w = 380.0;
                let card_h = if self.unlock_error.is_some() { 255.0 } else { 230.0 };
                let card_rect = egui::Rect::from_center_size(area.center(), vec2(card_w, card_h));
                let painter = ui.painter_at(area);

                painter.rect_filled(
                    card_rect.translate(vec2(0.0, 6.0)),
                    CornerRadius::same(12),
                    Color32::from_black_alpha(30),
                );
                painter.rect_filled(
                    card_rect,
                    CornerRadius::same(12),
                    Color32::from_rgba_unmultiplied(0xF4, 0xF7, 0xF8, 235),
                );
                painter.rect_stroke(
                    card_rect,
                    CornerRadius::same(12),
                    Stroke::new(1.0_f32, theme::shade(22)),
                    egui::StrokeKind::Inside,
                );

                let mut card_ui = ui.new_child(egui::UiBuilder::new().max_rect(card_rect.shrink(28.0)));
                card_ui.vertical_centered(|ui| {
                    ui.label(
                        RichText::new(s.unlock_title)
                            .font(theme::semibold(17.0))
                            .color(theme::TEXT_PRIMARY),
                    );
                    ui.add_space(6.0);
                    ui.label(
                        RichText::new(s.unlock_desc)
                            .font(theme::regular(12.5))
                            .color(theme::TEXT_MUTED),
                    );
                    ui.add_space(18.0);

                    let field_w = ui.available_width();
                    let (field_rect, _) = ui.allocate_exact_size(vec2(field_w, 36.0), egui::Sense::hover());
                    ui.painter().rect_filled(
                        field_rect,
                        CornerRadius::same(8),
                        Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, 220),
                    );
                    let edit_rect = egui::Rect::from_min_max(
                        egui::pos2(field_rect.left() + 12.0, field_rect.top()),
                        egui::pos2(field_rect.right() - 34.0, field_rect.bottom()),
                    );
                    let pw_resp = ui.put(
                        edit_rect,
                        egui::TextEdit::singleline(&mut self.unlock_password)
                            .id_salt("unlock_pw_input")
                            .password(!self.unlock_show_password)
                            .frame(false)
                            .font(theme::regular(13.0))
                            .text_color(theme::TEXT_PRIMARY)
                            .vertical_align(egui::Align::Center)
                            .hint_text(
                                RichText::new(s.panel_password)
                                    .color(theme::TEXT_PLACEHOLDER)
                                    .font(theme::regular(13.0)),
                            )
                            .desired_width(edit_rect.width()),
                    );
                    let init_id = pw_resp.id.with("unlock_initial_focus");
                    let did_focus = ui.data_mut(|d| {
                        let already = d.get_temp::<bool>(init_id).unwrap_or(false);
                        if !already {
                            d.insert_temp(init_id, true);
                        }
                        already
                    });
                    if !did_focus {
                        pw_resp.request_focus();
                    }
                    let eye_r = egui::Rect::from_center_size(
                        egui::pos2(field_rect.right() - 18.0, field_rect.center().y),
                        vec2(26.0, 26.0),
                    );
                    let eye_resp = ui
                        .interact(eye_r, ui.id().with("unlock_pw_eye"), egui::Sense::click())
                        .on_hover_cursor(CursorIcon::PointingHand);
                    if eye_resp.hovered() {
                        ui.painter()
                            .rect_filled(eye_r, CornerRadius::same(6), theme::shade(12));
                    }
                    let eye_glyph = if self.unlock_show_password {
                        crate::ui::icons::regular::EYE_SLASH
                    } else {
                        crate::ui::icons::regular::EYE
                    };
                    crate::ui::icons::paint(
                        ui.painter(),
                        eye_r.center(),
                        eye_glyph,
                        14.5,
                        theme::TEXT_MUTED,
                        crate::ui::icons::IconStyle::Regular,
                    );
                    if eye_resp.clicked() {
                        self.unlock_show_password = !self.unlock_show_password;
                    }
                    let ring = if pw_resp.has_focus() {
                        Stroke::new(1.5_f32, theme::FOCUS_RING)
                    } else {
                        Stroke::new(1.0_f32, theme::shade(24))
                    };
                    ui.painter()
                        .rect_stroke(field_rect, CornerRadius::same(8), ring, egui::StrokeKind::Inside);

                    if let Some(err) = &self.unlock_error {
                        ui.add_space(8.0);
                        ui.label(RichText::new(err).color(theme::DANGER).size(12.5));
                    }

                    ui.add_space(16.0);
                    let btn = egui::Button::new(
                        RichText::new(s.unlock_btn)
                            .color(Color32::WHITE)
                            .font(theme::semibold(13.0)),
                    )
                    .fill(theme::ACCENT_BLUE)
                    .corner_radius(CornerRadius::same(8))
                    .min_size(vec2(field_w, 38.0));
                    let submitted = ui
                        .add(btn)
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                        || (pw_resp.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                    if submitted {
                        match load_sync_config(&self.sync_path, &self.unlock_password) {
                            Ok(cfg) => {
                                let auto = cfg.auto_sync;
                                let pw = std::mem::take(&mut self.unlock_password);
                                self.sync_cfg = Some(cfg);
                                self.master_password = Some(pw);
                                self.unlock_error = None;
                                self.sync_locked = false;
                                if auto {
                                    self.spawn_pull(ctx.clone());
                                }
                            }
                            Err(e) => {
                                self.unlock_error = Some(format!("{e}"));
                            }
                        }
                    }
                });
            });
            handle_window_resize(ctx);
            return;
        }
        match tb_action {
            TitlebarAction::Select(ActiveTab::Settings) => self.settings_modal.open = true,
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
                .show(ctx, |ui| render_sidebar(ui, &mut self.nav, self.app_settings.language));
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
                        panel_action = render_host_panel(ui, m, self.app_settings.language);
                    });

                match panel_action {
                    HostModalAction::Save(host) => {
                        if let Some(existing) = self.vault.hosts.iter_mut().find(|h| h.id == host.id) {
                            *existing = host;
                        } else {
                            self.vault.hosts.push(host);
                        }
                        self.persist_vault(Some(ctx));
                        self.ghost_hosts = find_unimported_ssh_hosts(&self.vault);
                        self.panel_open = false;
                    }
                    HostModalAction::SaveAndConnect(host) => {
                        if let Some(existing) = self.vault.hosts.iter_mut().find(|h| h.id == host.id) {
                            *existing = host.clone();
                        } else {
                            self.vault.hosts.push(host.clone());
                        }
                        self.persist_vault(Some(ctx));
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
                        render_terminal(
                            ui,
                            &mut tab.terminal,
                            &tab.ssh.cmd_tx,
                            self.app_settings.text_size,
                            self.app_settings.copy_on_select,
                            self.app_settings.bright_bold,
                        );
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
                            self.app_settings.language,
                        ) {
                            ConnectingAction::Close => close_tab_id = Some(tab.id),
                            ConnectingAction::ToggleLogs => tab.show_logs = !tab.show_logs,
                            ConnectingAction::None => {}
                        }
                    }
                }
            }
            ActiveTab::Sftp => placeholder(ui, &format!("{} — {}", s.tab_sftp, s.coming_soon)),
            ActiveTab::Settings => placeholder(ui, &format!("{} — {}", s.menu_settings, s.coming_soon)),
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
                        &mut self.new_host_menu_open,
                        self.app_settings.language,
                    );
                }
                NavSection::Keychain => placeholder(ui, &format!("{} — {}", s.nav_keychain, s.coming_soon)),
                NavSection::PortForwarding => placeholder(ui, &format!("{} — {}", s.nav_port_forwarding, s.coming_soon)),
                NavSection::Snippets => placeholder(ui, &format!("{} — {}", s.nav_snippets, s.coming_soon)),
                NavSection::KnownHosts => placeholder(ui, &format!("{} — {}", s.nav_known_hosts, s.coming_soon)),
                NavSection::Logs => placeholder(ui, &format!("{} — {}", s.nav_logs, s.coming_soon)),
            },
        });
        self.handle_host_action(host_action, ctx);
        if let Some(cid) = close_tab_id {
            self.close_session(cid);
        }

        let (sync_action, settings_changed) = render_settings_modal(
            ctx,
            &mut self.settings_modal,
            &mut self.app_settings,
            self.sync_cfg.as_ref(),
        );
        if settings_changed {
            let _ = save_app_settings(&self.settings_path, &self.app_settings);
        }
        match sync_action {
            SyncUiAction::CreateNew { token, password } => {
                self.spawn_create_sync(token, password, ctx.clone());
            }
            SyncUiAction::ConnectExisting { token, password } => {
                self.spawn_connect_existing(token, password, ctx.clone());
            }
            SyncUiAction::SyncNow => {
                self.spawn_pull(ctx.clone());
            }
            SyncUiAction::ToggleAutoSync(enabled) => {
                if let (Some(cfg), Some(pw)) = (&mut self.sync_cfg, &self.master_password) {
                    cfg.auto_sync = enabled;
                    let _ = save_sync_config(&self.sync_path, cfg, pw);
                }
            }
            SyncUiAction::ToggleAutoUnlock(enabled) => {
                self.app_settings.auto_unlock_password = if enabled {
                    self.master_password.clone()
                } else {
                    None
                };
                let _ = save_app_settings(&self.settings_path, &self.app_settings);
            }
            SyncUiAction::Disconnect => {
                let _ = std::fs::remove_file(&self.sync_path);
                self.sync_cfg = None;
                self.master_password = None;
                self.app_settings.auto_unlock_password = None;
                let _ = save_app_settings(&self.settings_path, &self.app_settings);
                self.settings_modal.sync_step = SyncOnboardingStep::ChooseMode;
                self.settings_modal.sync_status = None;
                self.settings_modal.sync_error = None;
            }
            SyncUiAction::None => {}
        }
        handle_window_resize(ctx);
    }
}
