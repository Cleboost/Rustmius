use gtk4::prelude::*;
use gtk4::{glib, gio, gdk};
use crate::ui::server_list::{ServerList, ServerAction};
use crate::ui::add_server_dialog::show_server_dialog;
use crate::ui::file_explorer::FileExplorer;
use crate::ui::monitor::SystemMonitor;
use crate::ui::ssh_keys::build_ssh_keys_ui;
use crate::config_observer::{add_host_to_config, delete_host_from_config, load_hosts};
use vte4::prelude::*;
use std::rc::Rc;
use std::cell::RefCell;

pub fn build_ui(app: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Rustmius")
        .default_width(1100)
        .default_height(800)
        .build();

    let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    sidebar.set_width_request(60);
    sidebar.set_margin_top(12);
    let btn_servers = gtk4::Button::from_icon_name("network-transmit-receive-symbolic");
    btn_servers.add_css_class("flat");
    btn_servers.set_halign(gtk4::Align::Center);
    btn_servers.set_valign(gtk4::Align::Start);
    btn_servers.set_width_request(36);
    btn_servers.set_height_request(36);
    let btn_keys = gtk4::Button::from_icon_name("changes-prevent-symbolic");
    btn_keys.add_css_class("flat");
    btn_keys.set_halign(gtk4::Align::Center);
    btn_keys.set_valign(gtk4::Align::Start);
    btn_keys.set_width_request(36);
    btn_keys.set_height_request(36);
    let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    let btn_settings = gtk4::Button::from_icon_name("applications-system-symbolic");
    btn_settings.add_css_class("flat");
    btn_settings.set_halign(gtk4::Align::Center);
    btn_settings.set_valign(gtk4::Align::Start);
    btn_settings.set_width_request(36);
    btn_settings.set_height_request(36);
    btn_settings.set_margin_bottom(12);
    sidebar.append(&btn_servers);
    sidebar.append(&btn_keys);
    sidebar.append(&spacer);
    sidebar.append(&btn_settings);

    let separator = gtk4::Separator::new(gtk4::Orientation::Vertical);

    let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content_box.set_hexpand(true);
    let header = gtk4::HeaderBar::new();
    let add_btn = gtk4::Button::from_icon_name("list-add-symbolic");
    add_btn.add_css_class("suggested-action");
    header.pack_start(&add_btn);
    window.set_titlebar(Some(&header));

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

    let last_session_page = Rc::new(RefCell::new(0u32));

    let last_pg = last_session_page.clone();

    let refresh_ui: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let stack_clone = stack.clone();
    let window_clone = window.clone();
    let notebook_clone = notebook.clone();
    let refresh_ui_weak = Rc::downgrade(&refresh_ui);

    notebook.connect_switch_page(move |nb, _, _| {
        *last_pg.borrow_mut() = nb.current_page().unwrap_or(0);
    });

    notebook.connect_page_reordered(|nb, child, page_num| {
        if page_num == 0 && child.widget_name() != "server_list_tab" {
            nb.reorder_child(child, Some(1));
        }
    });

    let do_refresh = {
        let sc = stack_clone.clone();
        let wc = window_clone.clone();
        let nc = notebook_clone.clone();
        let rwh = refresh_ui_weak.clone();
        Rc::new(move || {
            let stack = sc.clone();
            let window = wc.clone();
            let notebook = nc.clone();
            let refresh_ui_handle = rwh.upgrade().unwrap();
            let sl_stack = stack.clone();
            let sl_window = window.clone();
            let sl_notebook = notebook.clone();
            let sl_refresh_handle = refresh_ui_handle.clone();

        let sl = ServerList::new(move |action| {
            let stack = sl_stack.clone();
            let window = sl_window.clone();
            let notebook = sl_notebook.clone();
            let refresh = sl_refresh_handle.borrow().as_ref().unwrap().clone();
            match action {
                ServerAction::Connect(host) => {
                    stack.set_visible_child_name("sessions");
                    let session_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
                    let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                    toolbar.set_margin_top(4);
                    toolbar.set_margin_bottom(4);
                    toolbar.set_margin_start(6);
                    let explorer_btn = gtk4::Button::from_icon_name("folder-remote-symbolic");
                    explorer_btn.add_css_class("flat");
                    explorer_btn.set_tooltip_text(Some("File Explorer"));

                    let monitor_btn = gtk4::Button::from_icon_name("utilities-system-monitor-symbolic");
                    monitor_btn.add_css_class("flat");
                    monitor_btn.set_tooltip_text(Some("System Monitor"));

                    let docker_btn = gtk4::Button::from_icon_name("view-grid-symbolic");
                    docker_btn.add_css_class("flat");
                    docker_btn.set_tooltip_text(Some("Docker Management (WIP)"));

                    toolbar.append(&explorer_btn);
                    toolbar.append(&monitor_btn);
                    toolbar.append(&docker_btn);
                    session_box.append(&toolbar);

                    let terminal = vte4::Terminal::new();
                    terminal.set_vexpand(true);
                    
                    let app_config = crate::config_observer::load_app_config();
                    let font_desc = gtk4::pango::FontDescription::from_string(&app_config.terminal_font);
                    terminal.set_font(Some(&font_desc));
                    terminal.set_scrollback_lines(app_config.terminal_scrollback as i64);

                    let key_controller = gtk4::EventControllerKey::new();
                    let terminal_clone = terminal.clone();
                    key_controller.connect_key_pressed(move |_controller, keyval, _keycode, state| {
                        let is_ctrl = state.contains(gdk::ModifierType::CONTROL_MASK);
                        let is_shift = state.contains(gdk::ModifierType::SHIFT_MASK);

                        if is_ctrl && is_shift {
                            match keyval {
                                gdk::Key::C => {
                                    terminal_clone.copy_clipboard_format(vte4::Format::Text);
                                    glib::Propagation::Stop
                                }
                                gdk::Key::V => {
                                    terminal_clone.paste_clipboard();
                                    glib::Propagation::Stop
                                }
                                _ => glib::Propagation::Proceed,
                            }
                        } else {
                            glib::Propagation::Proceed
                        }
                    });
                    terminal.add_controller(key_controller);

                    session_box.append(&terminal);

                    let mut count = 0;
                    for i in 0..notebook.n_pages() {
                        if let Some(p) = notebook.nth_page(Some(i))
                            && p.widget_name().starts_with(&format!("session:{}", host.alias)) {
                                count += 1;
                            }
                    }
                    session_box.set_widget_name(&format!("session:{}:{}", host.alias, count));

                    let display_name = if count > 0 { format!("{} ({})", host.alias, count) } else { host.alias.clone() };
                    let tab_label_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                    let label = gtk4::Label::new(Some(&display_name));
                    let close_btn = gtk4::Button::from_icon_name("window-close-symbolic");
                    close_btn.add_css_class("flat");
                    tab_label_box.append(&label);
                    tab_label_box.append(&close_btn);
                    let mut insert_pos = notebook.n_pages();
                    for i in 0..notebook.n_pages() {
                        if let Some(c) = notebook.nth_page(Some(i))
                            && c.widget_name() == "plus_tab_dummy" { insert_pos = i; break; }
                    }
                    notebook.insert_page(&session_box, Some(&tab_label_box), Some(insert_pos));
                    notebook.set_tab_reorderable(&session_box, true);
                    notebook.set_current_page(Some(insert_pos));
                    let nb_close = notebook.clone();
                    let sb_close = session_box.clone();
                    let win_ref = window.clone();

                    let close_gesture = gtk4::GestureClick::new();
                    close_gesture.connect_pressed(move |gesture, _, _, _| {
                        gesture.set_state(gtk4::EventSequenceState::Claimed);
                        
                        let nb = nb_close.clone();
                        let sb = sb_close.clone();
                        let win = win_ref.clone();

                        let on_confirm = move || {
                            let idx = nb.page_num(&sb);
                            if let Some(i) = idx {
                                let current = nb.current_page();
                                if current == Some(i) && nb.n_pages() > 2 {
                                    let target = if i > 0 { i - 1 } else { 1 };
                                    nb.set_current_page(Some(target));
                                }
                                nb.remove_page(Some(i));
                            }
                        };

                        let app_config = crate::config_observer::load_app_config();
                        if app_config.confirm_tab_close {
                            show_close_confirmation(&win, "Close Tab?", "Are you sure you want to close this session?", on_confirm);
                        } else {
                            on_confirm();
                        }
                    });
                    close_btn.add_controller(close_gesture);

                    let nb_exp = notebook.clone();
                    let host_exp = host.clone();
                    let win_exp = window.clone();
                    explorer_btn.connect_clicked(move |_| {
                        let h_exp = host_exp.clone();
                        let h_alias = h_exp.alias.clone();
                        let nb_spawn = nb_exp.clone();
                        let window = win_exp.clone();

                        glib::MainContext::default().spawn_local(async move {
                            let mut password = None;
                            if let Ok(keyring) = oo7::Keyring::new().await {
                                let mut attr = std::collections::HashMap::new();
                                let alias_lower = h_alias.to_lowercase();
                                attr.insert("rustmius-server-alias", alias_lower.as_str());
                                if let Ok(items) = keyring.search_items(&attr).await
                                    && let Some(item) = items.first()
                                        && let Ok(pass) = item.secret().await {
                                            password = std::str::from_utf8(pass.as_ref()).map(String::from).ok();
                                        }
                            }

                            let explorer = FileExplorer::new(h_exp, password);

                            let mut count = 0;
                            for i in 0..nb_spawn.n_pages() {
                                if let Some(p) = nb_spawn.nth_page(Some(i))
                                    && p.widget_name().starts_with(&format!("explorer:{}", h_alias)) {
                                        count += 1;
                                    }
                            }
                            explorer.container.set_widget_name(&format!("explorer:{}:{}", h_alias, count));

                            let display_name = if count > 0 { format!("📁 {} ({})", h_alias, count) } else { format!("📁 {}", h_alias) };
                            let exp_tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                            exp_tab_box.append(&gtk4::Label::new(Some(&display_name)));
                            let exp_close = gtk4::Button::from_icon_name("window-close-symbolic");
                            exp_close.add_css_class("flat");
                            exp_tab_box.append(&exp_close);

                            let mut ins_pos = nb_spawn.n_pages();
                            for i in 0..nb_spawn.n_pages() {
                                if let Some(c) = nb_spawn.nth_page(Some(i))
                                    && c.widget_name() == "plus_tab_dummy" { ins_pos = i; break; }
                            }
                            nb_spawn.insert_page(&explorer.container, Some(&exp_tab_box), Some(ins_pos));
                            nb_spawn.set_tab_reorderable(&explorer.container, true);
                            nb_spawn.set_current_page(Some(ins_pos));

                            let nb_c = nb_spawn.clone();
                            let ex_c = explorer.container.clone();
                            let win_c = window.clone();

                            let exp_close_gesture = gtk4::GestureClick::new();
                            exp_close_gesture.connect_pressed(move |gesture, _, _, _| {
                                gesture.set_state(gtk4::EventSequenceState::Claimed);
                                let nb = nb_c.clone();
                                let ex = ex_c.clone();
                                let win = win_c.clone();

                                let on_confirm = move || {
                                    let idx = nb.page_num(&ex);
                                    if let Some(i) = idx {
                                        let current = nb.current_page();
                                        if current == Some(i) && nb.n_pages() > 2 {
                                            let target = if i > 0 { i - 1 } else { 1 };
                                            nb.set_current_page(Some(target));
                                        }
                                        nb.remove_page(Some(i));
                                    }
                                };

                                let app_config = crate::config_observer::load_app_config();
                                if app_config.confirm_tab_close {
                                    show_close_confirmation(&win, "Close Explorer?", "Are you sure you want to close this explorer tab?", on_confirm);
                                } else {
                                    on_confirm();
                                }
                            });
                            exp_close.add_controller(exp_close_gesture);
                        });
                    });

                    let nb_mon = notebook.clone();
                    let host_mon = host.clone();
                    let win_mon = window.clone();
                    monitor_btn.connect_clicked(move |_| {
                        let h_mon = host_mon.clone();
                        let h_alias = h_mon.alias.clone();
                        let nb_spawn = nb_mon.clone();
                        let window = win_mon.clone();

                        glib::MainContext::default().spawn_local(async move {
                            let mut password = None;
                            if let Ok(keyring) = oo7::Keyring::new().await {
                                let mut attr = std::collections::HashMap::new();
                                let alias_lower = h_alias.to_lowercase();
                                attr.insert("rustmius-server-alias", alias_lower.as_str());
                                if let Ok(items) = keyring.search_items(&attr).await
                                    && let Some(item) = items.first()
                                        && let Ok(pass) = item.secret().await {
                                            password = std::str::from_utf8(pass.as_ref()).map(String::from).ok();
                                        }
                            }

                            let monitor = SystemMonitor::new(h_mon, password);

                            let mut count = 0;
                            for i in 0..nb_spawn.n_pages() {
                                if let Some(p) = nb_spawn.nth_page(Some(i))
                                    && p.widget_name().starts_with(&format!("monitor:{}", h_alias)) {
                                        count += 1;
                                    }
                            }
                            monitor.container.set_widget_name(&format!("monitor:{}:{}", h_alias, count));

                            let display_name = if count > 0 { format!("📈 {} ({})", h_alias, count) } else { format!("📈 {}", h_alias) };
                            let mon_tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                            mon_tab_box.append(&gtk4::Label::new(Some(&display_name)));
                            let mon_close = gtk4::Button::from_icon_name("window-close-symbolic");
                            mon_close.add_css_class("flat");
                            mon_tab_box.append(&mon_close);

                            let mut ins_pos = nb_spawn.n_pages();
                            for i in 0..nb_spawn.n_pages() {
                                if let Some(c) = nb_spawn.nth_page(Some(i))
                                    && c.widget_name() == "plus_tab_dummy" { ins_pos = i; break; }
                            }
                            nb_spawn.insert_page(&monitor.container, Some(&mon_tab_box), Some(ins_pos));
                            nb_spawn.set_tab_reorderable(&monitor.container, true);
                            nb_spawn.set_current_page(Some(ins_pos));

                            let nb_c = nb_spawn.clone();
                            let mo_c = monitor.container.clone();
                            let win_c = window.clone();

                            let mon_close_gesture = gtk4::GestureClick::new();
                            mon_close_gesture.connect_pressed(move |gesture, _, _, _| {
                                gesture.set_state(gtk4::EventSequenceState::Claimed);
                                let nb = nb_c.clone();
                                let mo = mo_c.clone();
                                let win = win_c.clone();

                                let on_confirm = move || {
                                    let idx = nb.page_num(&mo);
                                    if let Some(i) = idx {
                                        let current = nb.current_page();
                                        if current == Some(i) && nb.n_pages() > 2 {
                                            let target = if i > 0 { i - 1 } else { 1 };
                                            nb.set_current_page(Some(target));
                                        }
                                        nb.remove_page(Some(i));
                                    }
                                };

                                let app_config = crate::config_observer::load_app_config();
                                if app_config.confirm_tab_close {
                                    show_close_confirmation(&win, "Close Monitor?", "Are you sure you want to close this monitoring tab?", on_confirm);
                                } else {
                                    on_confirm();
                                }
                            });
                            mon_close.add_controller(mon_close_gesture);
                        });
                    });

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
                    let mut ssh_args = vec![
                        "/usr/bin/ssh".to_string(),
                        "-p".to_string(),
                        port_str,
                        "-o".to_string(),
                        "StrictHostKeyChecking=no".to_string(),
                    ];
                    if let Some(identity_file) = &host.identity_file {
                        ssh_args.push("-i".to_string());
                        ssh_args.push(identity_file.clone());
                    }
                    ssh_args.push(format!("{}@{}", user_str, host_str));
                    let ssh_args_refs: Vec<&str> = ssh_args.iter().map(|s| s.as_str()).collect();

                    terminal.spawn_async(vte4::PtyFlags::DEFAULT, None, &ssh_args_refs, &env_refs, glib::SpawnFlags::SEARCH_PATH, || {}, -1, None::<&gio::Cancellable>, |_| {});
                },
                ServerAction::Delete(host) => {
                    let _ = delete_host_from_config(&host.alias);
                    let alias_norm = host.alias.to_lowercase();
                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(keyring) = oo7::Keyring::new().await {
                            let mut attr = std::collections::HashMap::new();
                            attr.insert("rustmius-server-alias", alias_norm.as_str());
                            if let Ok(items) = keyring.search_items(&attr).await { for item in items { let _ = item.delete().await; } }
                        }
                    });
                    refresh();
                },
                ServerAction::Edit(host) => {
                    let host_to_edit = host.clone();
                    let old_alias = host.alias.clone();
                    let refresh_edit = refresh.clone();
                    let existing_aliases: Vec<String> = load_hosts().into_iter().map(|h| h.alias.to_lowercase()).collect();
                    show_server_dialog(window.upcast_ref(), Some(&host_to_edit), existing_aliases, move |new_host, password| {
                        let _ = delete_host_from_config(&old_alias);
                        if let Ok(_) = add_host_to_config(&new_host) {
                            if !password.is_empty() {
                                let host_alias = new_host.alias.clone();
                                glib::MainContext::default().spawn_local(async move {
                                    if let Ok(keyring) = oo7::Keyring::new().await {
                                        let mut attr = std::collections::HashMap::new();
                                        let alias_lower = host_alias.to_lowercase();
                                        attr.insert("rustmius-server-alias", alias_lower.as_str());
                                        let _ = keyring.create_item(&format!("Rustmius: SSH Password for {}", host_alias), &attr, password.as_bytes(), true).await;
                                    }
                                });
                            }
                            refresh_edit();
                        }
                    });
                }
            }
        });
        let mut server_list_idx = None;
        for i in 0..notebook.n_pages() {
            if let Some(c) = notebook.nth_page(Some(i))
                && c.widget_name() == "server_list_tab" { server_list_idx = Some(i); break; }
        }
        sl.container.set_widget_name("server_list_tab");
        let tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        tab_box.append(&gtk4::Image::from_icon_name("view-grid-symbolic"));
        tab_box.append(&gtk4::Label::new(Some("Connect")));
        if let Some(idx) = server_list_idx {
            notebook.remove_page(Some(idx));
            notebook.insert_page(&sl.container, Some(&tab_box), Some(idx));
        } else {
            notebook.insert_page(&sl.container, Some(&tab_box), Some(0));
            notebook.set_current_page(Some(0));
        }
        notebook.set_tab_reorderable(&sl.container, false);

        })
    };

    *refresh_ui.borrow_mut() = Some(do_refresh.clone());
    do_refresh();

    let stack_sessions = stack.clone();
    let nb_sessions = notebook.clone();
    btn_servers.connect_clicked(move |_| {
        let mut sl_idx = None;
        for i in 0..nb_sessions.n_pages() {
            if let Some(c) = nb_sessions.nth_page(Some(i))
                && c.widget_name() == "server_list_tab" { sl_idx = Some(i); break; }
        }
        if let Some(idx) = sl_idx {
            nb_sessions.set_current_page(Some(idx));
        } else {
            let refresh_h = refresh_ui_weak.upgrade().unwrap();
            if let Some(r) = refresh_h.borrow().as_ref() { r(); }
        }
        stack_sessions.set_visible_child_name("sessions");
    });

    let stack_nav_keys = stack.clone();
    btn_keys.connect_clicked(move |_| { stack_nav_keys.set_visible_child_name("ssh_keys"); });
    let stack_nav_settings = stack.clone();
    btn_settings.connect_clicked(move |_| { stack_nav_settings.set_visible_child_name("settings"); });

    let keys_box = build_ssh_keys_ui(&window);
    stack.add_named(&keys_box, Some("ssh_keys"));

    let settings_box = build_settings_ui(&window);
    stack.add_named(&settings_box, Some("settings"));

    let window_add = window.clone();
    let refresh_add = do_refresh.clone();
    add_btn.connect_clicked(move |_| {
        let refresh = refresh_add.clone();
        let existing_hosts = load_hosts();
        let existing_aliases: Vec<String> = existing_hosts.iter().map(|h| h.alias.to_lowercase()).collect();
        show_server_dialog(window_add.upcast_ref(), None, existing_aliases, move |new_host, password| {
            if let Ok(_) = add_host_to_config(&new_host) {
                if !password.is_empty() {
                    let host_alias = new_host.alias.clone();
                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(keyring) = oo7::Keyring::new().await {
                            let mut attr = std::collections::HashMap::new();
                            let alias_lower = host_alias.to_lowercase();
                            attr.insert("rustmius-server-alias", alias_lower.as_str());
                            let _ = keyring.create_item(&format!("Rustmius: SSH Password for {}", host_alias), &attr, password.as_bytes(), true).await;
                        }
                    });
                }
                refresh();
            }
        });
    });

    root.append(&sidebar); root.append(&separator); root.append(&content_box);
    window.set_child(Some(&root)); window.present();
}

fn build_settings_ui(parent_window: &gtk4::ApplicationWindow) -> gtk4::Box {
    let config = crate::config_observer::load_app_config();
    let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    
    let scrolled = gtk4::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk4::PolicyType::Never)
        .build();
    
    let content = gtk4::Box::new(gtk4::Orientation::Vertical, 32);
    content.set_margin_top(48);
    content.set_margin_bottom(48);
    content.set_margin_start(48);
    content.set_margin_end(48);
    content.set_halign(gtk4::Align::Center);
    content.set_width_request(600);

    let header_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
    let title = gtk4::Label::builder()
        .label("Settings")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-1".to_string()])
        .build();
    let subtitle = gtk4::Label::builder()
        .label("Configure your global preferences")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["dim-label".to_string()])
        .build();
    header_box.append(&title);
    header_box.append(&subtitle);
    content.append(&header_box);

    let terminal_group = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    let terminal_title = gtk4::Label::builder()
        .label("Terminal")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-4".to_string()])
        .build();
    terminal_group.append(&terminal_title);

    let font_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let font_label = gtk4::Label::new(Some("Font"));
    font_label.set_hexpand(true);
    font_label.set_halign(gtk4::Align::Start);
    let font_button = gtk4::FontButton::with_font(&config.terminal_font);
    font_row.append(&font_label);
    font_row.append(&font_button);
    terminal_group.append(&font_row);

    let scrollback_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let scrollback_label = gtk4::Label::new(Some("Scrollback Lines"));
    scrollback_label.set_hexpand(true);
    scrollback_label.set_halign(gtk4::Align::Start);
    let scrollback_adj = gtk4::Adjustment::new(config.terminal_scrollback as f64, 100.0, 100000.0, 100.0, 1000.0, 0.0);
    let scrollback_spinner = gtk4::SpinButton::new(Some(&scrollback_adj), 1.0, 0);
    scrollback_row.append(&scrollback_label);
    scrollback_row.append(&scrollback_spinner);
    terminal_group.append(&scrollback_row);
    content.append(&terminal_group);

    let monitor_group = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    let monitor_title = gtk4::Label::builder()
        .label("System Monitor")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-4".to_string()])
        .build();
    monitor_group.append(&monitor_title);

    let refresh_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let refresh_label = gtk4::Label::new(Some("Default Refresh Rate"));
    refresh_label.set_hexpand(true);
    refresh_label.set_halign(gtk4::Align::Start);
    let refresh_dropdown = gtk4::DropDown::from_strings(&["1s", "3s", "5s", "10s"]);
    refresh_dropdown.set_selected(config.monitor_refresh_rate);
    refresh_row.append(&refresh_label);
    refresh_row.append(&refresh_dropdown);
    monitor_group.append(&refresh_row);
    content.append(&monitor_group);

    let ui_group = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    let ui_title = gtk4::Label::builder()
        .label("User Interface")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["title-4".to_string()])
        .build();
    ui_group.append(&ui_title);

    let confirm_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let confirm_label = gtk4::Label::new(Some("Confirm before closing tabs"));
    confirm_label.set_hexpand(true);
    confirm_label.set_halign(gtk4::Align::Start);
    let confirm_switch = gtk4::Switch::new();
    confirm_switch.set_active(config.confirm_tab_close);
    confirm_row.append(&confirm_label);
    confirm_row.append(&confirm_switch);
    ui_group.append(&confirm_row);
    content.append(&ui_group);

    let r_drop = refresh_dropdown.clone();
    let f_btn = font_button.clone();
    let s_spin = scrollback_spinner.clone();
    let c_switch = confirm_switch.clone();

    let save_config = move || {
        let mut new_config = crate::config_observer::AppConfig::default();
        new_config.monitor_refresh_rate = r_drop.selected();
        new_config.terminal_font = f_btn.font().map(|s| s.to_string()).unwrap_or_else(|| "Monospace 11".to_string());
        new_config.terminal_scrollback = s_spin.value() as u32;
        new_config.confirm_tab_close = c_switch.is_active();
        
        let _ = crate::config_observer::save_app_config(&new_config);
    };

    let save_fn = Rc::new(save_config);
    
    let s1 = save_fn.clone();
    refresh_dropdown.connect_selected_notify(move |_| { s1(); });
    
    let s2 = save_fn.clone();
    font_button.connect_font_set(move |_| { s2(); });
    
    let s3 = save_fn.clone();
    scrollback_spinner.connect_value_changed(move |_| { s3(); });
    
    let s5 = save_fn.clone();
    confirm_switch.connect_active_notify(move |_| { s5(); });

    scrolled.set_child(Some(&content));
    container.append(&scrolled);
    container
}

fn show_close_confirmation(parent: &gtk4::ApplicationWindow, title: &str, message: &str, on_confirm: impl FnOnce() + 'static) {
    let dialog = gtk4::MessageDialog::builder()
        .transient_for(parent)
        .modal(true)
        .buttons(gtk4::ButtonsType::OkCancel)
        .text(title)
        .secondary_text(message)
        .build();
    
    let on_confirm = std::cell::RefCell::new(Some(on_confirm));
    dialog.connect_response(move |d, response| {
        if response == gtk4::ResponseType::Ok {
            if let Some(callback) = on_confirm.borrow_mut().take() {
                callback();
            }
        }
        d.close();
    });
    dialog.present();
}