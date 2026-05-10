use gtk4::prelude::*;
use gtk4::{glib, gio, gdk};
use crate::ui::server_list::{ServerList, ServerAction};
use crate::ui::add_server_dialog::show_server_dialog;
use crate::ui::file_explorer::FileExplorer;
use crate::ui::monitor::SystemMonitor;
use crate::ui::docker::DockerManager;
use crate::ui::ssh_keys::build_ssh_keys_ui;
use crate::ui::components::sidebar::Sidebar;
use crate::ui::components::header::Header;
use crate::ui::components::settings::Settings;
use crate::config_observer::{add_host_to_config, delete_host_from_config, load_hosts, SshHost};
use vte4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

#[derive(Clone)]
pub struct AppWindow {
    inner: Rc<AppWindowInner>,
}

struct AppWindowInner {
    window: gtk4::ApplicationWindow,
    stack: gtk4::Stack,
    notebook: gtk4::Notebook,
}

impl AppWindow {
    pub fn new(app: &gtk4::Application) -> Self {
        let window = gtk4::ApplicationWindow::builder()
            .application(app)
            .title("Rustmius")
            .default_width(1100)
            .default_height(800)
            .build();

        let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        let sidebar = Sidebar::new();
        let separator = gtk4::Separator::new(gtk4::Orientation::Vertical);
        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        content_box.set_hexpand(true);

        let header = Header::new();
        window.set_titlebar(Some(&header.container));

        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        content_box.append(&stack);

        let notebook = gtk4::Notebook::new();
        notebook.set_vexpand(true);
        notebook.set_hexpand(true);
        notebook.set_scrollable(true);
        notebook.set_show_border(false);
        
        let sessions_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        sessions_box.append(&notebook);
        stack.add_named(&sessions_box, Some("sessions"));

        let settings = Settings::new();
        stack.add_named(&settings.container, Some("settings"));

        let keys_box = build_ssh_keys_ui(&window);
        stack.add_named(&keys_box, Some("ssh_keys"));

        root.append(&sidebar.container);
        root.append(&separator);
        root.append(&content_box);
        window.set_child(Some(&root));

        let app_window = Self {
            inner: Rc::new(AppWindowInner {
                window,
                stack,
                notebook,
            }),
        };

        app_window.setup_callbacks(sidebar, header);
        app_window.refresh();
        app_window.inner.window.present();
        app_window
    }

    fn setup_callbacks(&self, sidebar: Sidebar, header: Header) {
        let this = self.clone();
        sidebar.btn_servers.connect_clicked(move |_| {
            this.show_sessions();
        });

        let this = self.clone();
        sidebar.btn_keys.connect_clicked(move |_| {
            this.inner.stack.set_visible_child_name("ssh_keys");
        });

        let this = self.clone();
        sidebar.btn_settings.connect_clicked(move |_| {
            this.inner.stack.set_visible_child_name("settings");
        });

        let this = self.clone();
        header.add_btn.connect_clicked(move |_| {
            this.show_add_server_dialog();
        });

        self.inner.notebook.connect_page_reordered(|nb, child, page_num| {
            if page_num == 0 && child.widget_name() != "server_list_tab" {
                nb.reorder_child(child, Some(1));
            }
        });
    }

    fn show_sessions(&self) {
        let mut sl_idx = None;
        for i in 0..self.inner.notebook.n_pages() {
            if let Some(c) = self.inner.notebook.nth_page(Some(i))
                && c.widget_name() == "server_list_tab" { sl_idx = Some(i); break; }
        }
        if let Some(idx) = sl_idx {
            self.inner.notebook.set_current_page(Some(idx));
        } else {
            self.refresh();
        }
        self.inner.stack.set_visible_child_name("sessions");
    }

    fn show_add_server_dialog(&self) {
        let this = self.clone();
        let existing_hosts = load_hosts().unwrap_or_else(|e| {
            tracing::error!("Failed to load hosts: {}", e);
            Vec::new()
        });
        let existing_aliases: Vec<String> = existing_hosts.iter().map(|h| h.alias.to_lowercase()).collect();
        
        show_server_dialog(self.inner.window.upcast_ref(), None, existing_aliases, move |new_host, password| {
            if let Ok(_) = add_host_to_config(&new_host) {
                if !password.is_empty() {
                    let host_alias = new_host.alias.clone();
                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(keyring) = oo7::Keyring::new().await {
                            let mut attr = HashMap::new();
                            let alias_lower = host_alias.to_lowercase();
                            attr.insert("rustmius-server-alias", alias_lower.as_str());
                            let _ = keyring.create_item(&format!("Rustmius: SSH Password for {}", host_alias), &attr, password.as_bytes(), true).await;
                        }
                    });
                }
                this.refresh();
            }
        });
    }

    pub fn refresh(&self) {
        let this = self.clone();
        let sl = ServerList::new(move |action| {
            match action {
                ServerAction::Connect(host, password) => this.connect_to_server(host, password),
                ServerAction::Delete(host) => this.delete_server(host),
                ServerAction::Edit(host) => this.edit_server(host),
            }
        });

        let mut server_list_idx = None;
        for i in 0..self.inner.notebook.n_pages() {
            if let Some(c) = self.inner.notebook.nth_page(Some(i))
                && c.widget_name() == "server_list_tab" { server_list_idx = Some(i); break; }
        }

        sl.container.set_widget_name("server_list_tab");
        let tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        tab_box.append(&gtk4::Image::from_icon_name("view-grid-symbolic"));
        tab_box.append(&gtk4::Label::new(Some("Connect")));

        if let Some(idx) = server_list_idx {
            self.inner.notebook.remove_page(Some(idx));
            self.inner.notebook.insert_page(&sl.container, Some(&tab_box), Some(idx));
        } else {
            self.inner.notebook.insert_page(&sl.container, Some(&tab_box), Some(0));
            self.inner.notebook.set_current_page(Some(0));
        }
        self.inner.notebook.set_tab_reorderable(&sl.container, false);
    }

    fn connect_to_server(&self, host: SshHost, password: Option<String>) {
        self.inner.stack.set_visible_child_name("sessions");
        let session_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        toolbar.set_margin_top(4); toolbar.set_margin_bottom(4); toolbar.set_margin_start(6);

        let explorer_btn = gtk4::Button::from_icon_name("folder-remote-symbolic");
        explorer_btn.add_css_class("flat");
        explorer_btn.set_tooltip_text(Some("File Explorer"));

        let monitor_btn = gtk4::Button::from_icon_name("utilities-system-monitor-symbolic");
        monitor_btn.add_css_class("flat");
        monitor_btn.set_tooltip_text(Some("System Monitor"));

        let docker_btn = gtk4::Button::new();
        docker_btn.set_child(Some(&crate::ui::get_docker_icon()));
        docker_btn.add_css_class("flat");
        docker_btn.set_tooltip_text(Some("Docker Management"));

        toolbar.append(&explorer_btn);
        toolbar.append(&monitor_btn);
        toolbar.append(&docker_btn);
        session_box.append(&toolbar);

        let terminal = vte4::Terminal::new();
        terminal.set_vexpand(true);
        let app_config = crate::config_observer::load_app_config().unwrap_or_else(|e| {
            tracing::error!("Failed to load app config: {}", e);
            crate::config_observer::AppConfig::default()
        });
        let font_desc = gtk4::pango::FontDescription::from_string(&app_config.terminal_font);
        terminal.set_font(Some(&font_desc));
        terminal.set_scrollback_lines(app_config.terminal_scrollback as i64);

        let key_controller = gtk4::EventControllerKey::new();
        let terminal_clone = terminal.clone();
        key_controller.connect_key_pressed(move |_, keyval, _, state| {
            let is_ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
            let is_shift = state.contains(gdk::ModifierType::SHIFT_MASK);
            if is_ctrl && is_shift {
                match keyval {
                    gdk::Key::C => { terminal_clone.copy_clipboard_format(vte4::Format::Text); glib::Propagation::Stop }
                    gdk::Key::V => { terminal_clone.paste_clipboard(); glib::Propagation::Stop }
                    _ => glib::Propagation::Proceed,
                }
            } else { glib::Propagation::Proceed }
        });
        terminal.add_controller(key_controller);
        session_box.append(&terminal);

        let mut count = 0;
        for i in 0..self.inner.notebook.n_pages() {
            if let Some(p) = self.inner.notebook.nth_page(Some(i))
                && p.widget_name().starts_with(&format!("session:{}", host.alias)) { count += 1; }
        }
        session_box.set_widget_name(&format!("session:{}:{}", host.alias, count));

        let display_name = if count > 0 { format!("{} ({})", host.alias, count) } else { host.alias.clone() };
        let tab_label_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        let label = gtk4::Label::new(Some(&display_name));
        let close_btn = gtk4::Button::from_icon_name("window-close-symbolic");
        close_btn.add_css_class("flat");
        tab_label_box.append(&label);
        tab_label_box.append(&close_btn);

        let mut insert_pos = self.inner.notebook.n_pages();
        for i in 0..self.inner.notebook.n_pages() {
            if let Some(c) = self.inner.notebook.nth_page(Some(i))
                && c.widget_name() == "plus_tab_dummy" { insert_pos = i; break; }
        }
        self.inner.notebook.insert_page(&session_box, Some(&tab_label_box), Some(insert_pos));
        self.inner.notebook.set_tab_reorderable(&session_box, true);
        self.inner.notebook.set_current_page(Some(insert_pos));

        let notebook = self.inner.notebook.clone();
        let win = self.inner.window.clone();
        close_btn.connect_clicked(move |_| {
            let nb = notebook.clone();
            let sb = session_box.clone();
            let w = win.clone();
            let on_confirm = move || {
                if let Some(i) = nb.page_num(&sb) {
                    nb.remove_page(Some(i));
                }
            };
            if crate::config_observer::load_app_config().map(|c| c.confirm_tab_close).unwrap_or(false) {
                show_close_confirmation(w.upcast_ref(), "Close Tab?", "Are you sure you want to close this session?", on_confirm);
            } else { on_confirm(); }
        });

        let notebook_exp = self.inner.notebook.clone();
        let win_exp = self.inner.window.clone();
        let host_exp = host.clone();
        explorer_btn.connect_clicked(move |_| {
            Self::spawn_explorer(&notebook_exp, &win_exp, host_exp.clone());
        });

        let notebook_mon = self.inner.notebook.clone();
        let win_mon = self.inner.window.clone();
        let host_mon = host.clone();
        monitor_btn.connect_clicked(move |_| {
            Self::spawn_monitor(&notebook_mon, &win_mon, host_mon.clone());
        });

        let notebook_docker = self.inner.notebook.clone();
        let host_docker = host.clone();
        let pass_docker = password.clone();
        docker_btn.connect_clicked(move |_| {
            Self::spawn_docker(&notebook_docker, host_docker.clone(), pass_docker.clone());
        });

        self.spawn_ssh_process(&terminal, &host);
    }

    fn spawn_ssh_process(&self, terminal: &vte4::Terminal, host: &SshHost) {
        let host_str = host.hostname.clone();
        let user_str = host.user.clone().unwrap_or_else(|| "root".to_string());
        let host_alias = host.alias.clone();
        let exe_path = std::env::current_exe().unwrap_or_default().to_string_lossy().to_string();
        let mut envv: Vec<String> = std::env::vars().map(|(k, v)| format!("{}={}", k, v)).collect();
        if host.identity_file.is_none() {
            envv.push(format!("SSH_ASKPASS={}", exe_path));
            envv.push("SSH_ASKPASS_REQUIRE=force".to_string());
            envv.push(format!("RUSTMIUS_ASKPASS_ALIAS={}", host_alias));
        }
        envv.push("DISPLAY=:0".to_string());
        let env_refs: Vec<&str> = envv.iter().map(|s| s.as_str()).collect();
        let port_str = host.port.unwrap_or(22).to_string();
        let mut ssh_args = vec!["/usr/bin/ssh".to_string(), "-p".to_string(), port_str, "-o".to_string(), "StrictHostKeyChecking=no".to_string()];
        if let Some(identity_file) = &host.identity_file {
            ssh_args.push("-i".to_string()); ssh_args.push(identity_file.clone());
        }
        ssh_args.push(format!("{}@{}", user_str, host_str));
        let ssh_args_refs: Vec<&str> = ssh_args.iter().map(|s| s.as_str()).collect();
        terminal.spawn_async(vte4::PtyFlags::DEFAULT, None, &ssh_args_refs, &env_refs, glib::SpawnFlags::SEARCH_PATH, || {}, -1, None::<&gio::Cancellable>, |_| {});
    }

    fn spawn_explorer(notebook: &gtk4::Notebook, win: &gtk4::ApplicationWindow, host: SshHost) {
        let h_alias = host.alias.clone();
        let nb = notebook.clone();
        let window = win.clone();
        glib::MainContext::default().spawn_local(async move {
            let password = Self::get_keyring_password(&h_alias).await;
            let explorer = FileExplorer::new(host, password);
            let count = Self::count_pages_with_prefix(&nb, &format!("explorer:{}", h_alias));
            explorer.container.set_widget_name(&format!("explorer:{}:{}", h_alias, count));

            let display_name = if count > 0 { format!("📁 {} ({})", h_alias, count) } else { format!("📁 {}", h_alias) };
            
            let nb_inner = nb.clone();
            let ex_inner = explorer.container.clone();
            let win_inner = window.clone();
            let tab_box = Self::create_tab_label(&display_name, move || {
                let nb_confirm = nb_inner.clone();
                let ex_confirm = ex_inner.clone();
                let on_confirm = move || {
                    if let Some(i) = nb_confirm.page_num(&ex_confirm) { nb_confirm.remove_page(Some(i)); }
                };
                if crate::config_observer::load_app_config().map(|c| c.confirm_tab_close).unwrap_or(false) {
                    show_close_confirmation(win_inner.upcast_ref(), "Close Explorer?", "Are you sure you want to close this explorer tab?", on_confirm);
                } else { on_confirm(); }
            });

            let ins_pos = Self::get_insert_position(&nb);
            nb.insert_page(&explorer.container, Some(&tab_box), Some(ins_pos));
            nb.set_tab_reorderable(&explorer.container, true);
            nb.set_current_page(Some(ins_pos));
        });
    }

    fn spawn_monitor(notebook: &gtk4::Notebook, win: &gtk4::ApplicationWindow, host: SshHost) {
        let h_alias = host.alias.clone();
        let nb = notebook.clone();
        let window = win.clone();
        glib::MainContext::default().spawn_local(async move {
            let password = Self::get_keyring_password(&h_alias).await;
            let monitor = SystemMonitor::new(host, password);
            let count = Self::count_pages_with_prefix(&nb, &format!("monitor:{}", h_alias));
            monitor.container.set_widget_name(&format!("monitor:{}:{}", h_alias, count));

            let display_name = if count > 0 { format!("📈 {} ({})", h_alias, count) } else { format!("📈 {}", h_alias) };
            
            let nb_inner = nb.clone();
            let mo_inner = monitor.container.clone();
            let win_inner = window.clone();
            let tab_box = Self::create_tab_label(&display_name, move || {
                let nb_confirm = nb_inner.clone();
                let mo_confirm = mo_inner.clone();
                let on_confirm = move || {
                    if let Some(i) = nb_confirm.page_num(&mo_confirm) { nb_confirm.remove_page(Some(i)); }
                };
                if crate::config_observer::load_app_config().map(|c| c.confirm_tab_close).unwrap_or(false) {
                    show_close_confirmation(win_inner.upcast_ref(), "Close Monitor?", "Are you sure you want to close this monitoring tab?", on_confirm);
                } else { on_confirm(); }
            });

            let ins_pos = Self::get_insert_position(&nb);
            nb.insert_page(&monitor.container, Some(&tab_box), Some(ins_pos));
            nb.set_tab_reorderable(&monitor.container, true);
            nb.set_current_page(Some(ins_pos));
        });
    }

    fn spawn_docker(notebook: &gtk4::Notebook, host: SshHost, password: Option<String>) {
        let docker = DockerManager::new(host.clone(), password);
        let label_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        label_box.append(&crate::ui::get_docker_icon());
        label_box.append(&gtk4::Label::new(Some(&format!("Docker: {}", host.alias))));
        let close_btn = gtk4::Button::from_icon_name("window-close-symbolic");
        close_btn.add_css_class("flat");
        label_box.append(&close_btn);
        
        let page_idx = notebook.append_page(&docker.container, Some(&label_box));
        notebook.set_tab_reorderable(&docker.container, true);
        notebook.set_current_page(Some(page_idx));

        let nb_close = notebook.clone();
        let child_close = docker.container.clone();
        close_btn.connect_clicked(move |_| {
            if let Some(i) = nb_close.page_num(&child_close) { nb_close.remove_page(Some(i)); }
        });
    }

    fn delete_server(&self, host: SshHost) {
        let _ = delete_host_from_config(&host.alias);
        let alias_norm = host.alias.to_lowercase();
        let this = self.clone();
        glib::MainContext::default().spawn_local(async move {
            if let Ok(keyring) = oo7::Keyring::new().await {
                let mut attr = HashMap::new();
                attr.insert("rustmius-server-alias", alias_norm.as_str());
                if let Ok(items) = keyring.search_items(&attr).await {
                    for item in items { let _ = item.delete().await; }
                }
            }
            this.refresh();
        });
    }

    fn edit_server(&self, host: SshHost) {
        let old_alias = host.alias.clone();
        let this = self.clone();
        let existing_aliases: Vec<String> = load_hosts().unwrap_or_else(|e| {
            tracing::error!("Failed to load hosts: {}", e);
            Vec::new()
        }).into_iter().map(|h| h.alias.to_lowercase()).collect();
        show_server_dialog(self.inner.window.upcast_ref(), Some(&host), existing_aliases, move |new_host, password| {
            let _ = delete_host_from_config(&old_alias);
            if let Ok(_) = add_host_to_config(&new_host) {
                if !password.is_empty() {
                    let host_alias = new_host.alias.clone();
                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(keyring) = oo7::Keyring::new().await {
                            let mut attr = HashMap::new();
                            let alias_lower = host_alias.to_lowercase();
                            attr.insert("rustmius-server-alias", alias_lower.as_str());
                            let _ = keyring.create_item(&format!("Rustmius: SSH Password for {}", host_alias), &attr, password.as_bytes(), true).await;
                        }
                    });
                }
                this.refresh();
            }
        });
    }

    // Helper methods
    async fn get_keyring_password(alias: &str) -> Option<String> {
        if let Ok(keyring) = oo7::Keyring::new().await {
            let mut attr = HashMap::new();
            let alias_lower = alias.to_lowercase();
            attr.insert("rustmius-server-alias", alias_lower.as_str());
            if let Ok(items) = keyring.search_items(&attr).await
                && let Some(item) = items.first()
                && let Ok(pass) = item.secret().await {
                return std::str::from_utf8(pass.as_ref()).map(String::from).ok();
            }
        }
        None
    }

    fn count_pages_with_prefix(notebook: &gtk4::Notebook, prefix: &str) -> u32 {
        let mut count = 0;
        for i in 0..notebook.n_pages() {
            if let Some(p) = notebook.nth_page(Some(i))
                && p.widget_name().starts_with(prefix) { count += 1; }
        }
        count
    }

    fn get_insert_position(notebook: &gtk4::Notebook) -> u32 {
        for i in 0..notebook.n_pages() {
            if let Some(c) = notebook.nth_page(Some(i))
                && c.widget_name() == "plus_tab_dummy" { return i; }
        }
        notebook.n_pages()
    }

    fn create_tab_label(text: &str, on_close: impl Fn() + 'static) -> gtk4::Box {
        let tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        tab_box.append(&gtk4::Label::new(Some(text)));
        let close_btn = gtk4::Button::from_icon_name("window-close-symbolic");
        close_btn.add_css_class("flat");
        close_btn.connect_clicked(move |_| on_close());
        tab_box.append(&close_btn);
        tab_box
    }
}

pub fn build_ui(app: &gtk4::Application) {
    AppWindow::new(app);
}

fn show_close_confirmation(parent: &gtk4::Window, title: &str, message: &str, on_confirm: impl FnOnce() + 'static) {
    let dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(title)
        .detail(message)
        .buttons(vec!["Cancel", "OK"])
        .cancel_button(0)
        .default_button(1)
        .build();

    let on_confirm = RefCell::new(Some(on_confirm));
    dialog.choose(Some(parent), None::<&gio::Cancellable>, move |res| {
        if let Ok(idx) = res {
            if idx == 1 {
                if let Some(callback) = on_confirm.borrow_mut().take() { callback(); }
            }
        }
    });
}