use gtk4::prelude::*;
use gtk4::{glib, gio};
use crate::ui::server_list::{ServerList, ServerAction};
use crate::ui::add_server_dialog::show_server_dialog;
use crate::ui::file_explorer::FileExplorer;
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
    let stack = gtk4::Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
    content_box.append(&header);
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
                    monitor_btn.set_tooltip_text(Some("System Monitor (WIP)"));

                    let docker_btn = gtk4::Button::from_icon_name("view-grid-symbolic");
                    docker_btn.add_css_class("flat");
                    docker_btn.set_tooltip_text(Some("Docker Management (WIP)"));

                    toolbar.append(&explorer_btn);
                    toolbar.append(&monitor_btn);
                    toolbar.append(&docker_btn);
                    session_box.append(&toolbar);

                    let terminal = vte4::Terminal::new();
                    terminal.set_vexpand(true);
                    session_box.append(&terminal);

                    let tab_label_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                    let label = gtk4::Label::new(Some(&host.alias));
                    let close_btn = gtk4::Button::from_icon_name("window-close-symbolic");
                    close_btn.add_css_class("flat");
                    tab_label_box.append(&label);
                    tab_label_box.append(&close_btn);
                    
                    let mut insert_pos = notebook.n_pages();
                    for i in 0..notebook.n_pages() {
                        if let Some(c) = notebook.nth_page(Some(i)) {
                            if c.widget_name() == "plus_tab_dummy" { insert_pos = i; break; }
                        }
                    }
                    notebook.insert_page(&session_box, Some(&tab_label_box), Some(insert_pos));
                    notebook.set_tab_reorderable(&session_box, true);
                    notebook.set_current_page(Some(insert_pos));
                    
                    let nb_close = notebook.clone();
                    let sb_close = session_box.clone();

                    close_btn.connect_clicked(move |_| {
                        let idx = nb_close.page_num(&sb_close);
                        if let Some(i) = idx {
                            let current = nb_close.current_page();
                            if current == Some(i) && nb_close.n_pages() > 2 {
                                let target = if i > 0 { i - 1 } else { 1 };
                                nb_close.set_current_page(Some(target));
                            }
                            nb_close.remove_page(Some(i));
                        }
                    });

                    let nb_exp = notebook.clone();
                    let host_exp = host.clone();
                    explorer_btn.connect_clicked(move |_| {
                        let h_exp = host_exp.clone();
                        let h_alias = h_exp.alias.clone();
                        let nb_spawn = nb_exp.clone();

                        
                        glib::MainContext::default().spawn_local(async move {
                            let mut password = None;
                            if let Ok(keyring) = oo7::Keyring::new().await {
                                let mut attr = std::collections::HashMap::new();
                                let alias_lower = h_alias.to_lowercase();
                                attr.insert("rustmius-server-alias", alias_lower.as_str());
                                if let Ok(items) = keyring.search_items(&attr).await {
                                    if let Some(item) = items.first() {
                                        if let Ok(pass) = item.secret().await {
                                            password = Some(String::from_utf8_lossy(&pass).to_string());
                                        }
                                    }
                                }
                            }

                            let explorer = FileExplorer::new(h_exp, password);
                            let exp_tab_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                            exp_tab_box.append(&gtk4::Label::new(Some(&format!("📁 {}", h_alias))));
                            let exp_close = gtk4::Button::from_icon_name("window-close-symbolic");
                            exp_close.add_css_class("flat");
                            exp_tab_box.append(&exp_close);

                            let mut ins_pos = nb_spawn.n_pages();
                            for i in 0..nb_spawn.n_pages() {
                                if let Some(c) = nb_spawn.nth_page(Some(i)) {
                                    if c.widget_name() == "plus_tab_dummy" { ins_pos = i; break; }
                                }
                            }
                            nb_spawn.insert_page(&explorer.container, Some(&exp_tab_box), Some(ins_pos));
                            nb_spawn.set_current_page(Some(ins_pos));

                            let nb_c = nb_spawn.clone();
                            let ex_c = explorer.container.clone();
                            exp_close.connect_clicked(move |_| {
                                let idx = nb_c.page_num(&ex_c);
                                if let Some(i) = idx {
                                    let current = nb_c.current_page();
                                    if current == Some(i) && nb_c.n_pages() > 2 {
                                        let target = if i > 0 { i - 1 } else { 1 };
                                        nb_c.set_current_page(Some(target));
                                    }
                                    nb_c.remove_page(Some(i));
                                }
                            });
                        });
                    });



                    let host_str = host.hostname.clone();
                    let user_str = host.user.clone().unwrap_or_else(|| "root".to_string());
                    let host_alias = host.alias.clone();
                    let exe_path = std::env::current_exe().unwrap_or_default().to_string_lossy().to_string();
                    let mut envv: Vec<String> = std::env::vars().map(|(k, v)| format!("{}={}", k, v)).collect();
                    envv.push(format!("SSH_ASKPASS={}", exe_path));
                    envv.push("SSH_ASKPASS_REQUIRE=force".to_string());
                    envv.push(format!("RUSTMIUS_ASKPASS_ALIAS={}", host_alias));
                    envv.push("DISPLAY=:0".to_string());
                    let env_refs: Vec<&str> = envv.iter().map(|s| s.as_str()).collect();
                    terminal.spawn_async(vte4::PtyFlags::DEFAULT, None, &["/usr/bin/ssh", "-o", "StrictHostKeyChecking=no", "-o", "PubkeyAuthentication=no", &format!("{}@{}", user_str, host_str)], &env_refs, glib::SpawnFlags::SEARCH_PATH, || {}, -1, None::<&gio::Cancellable>, |_| {});
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
            if let Some(c) = notebook.nth_page(Some(i)) {
                if c.widget_name() == "server_list_tab" { server_list_idx = Some(i); break; }
            }
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

        })
    };

    *refresh_ui.borrow_mut() = Some(do_refresh.clone());
    do_refresh();

    let stack_sessions = stack.clone();
    let nb_sessions = notebook.clone();
    btn_servers.connect_clicked(move |_| {
        let mut sl_idx = None;
        for i in 0..nb_sessions.n_pages() {
            if let Some(c) = nb_sessions.nth_page(Some(i)) {
                if c.widget_name() == "server_list_tab" { sl_idx = Some(i); break; }
            }
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

    let keys_box = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
    keys_box.set_margin_top(48); keys_box.set_margin_bottom(48); keys_box.set_margin_start(48); keys_box.set_margin_end(48);
    keys_box.set_halign(gtk4::Align::Center); keys_box.set_valign(gtk4::Align::Center);
    let wip_icon = gtk4::Image::from_icon_name("system-shutdown-symbolic");
    wip_icon.set_pixel_size(96); wip_icon.add_css_class("dim-label");
    let wip_label = gtk4::Label::new(Some("SSH Keys Management - WIP"));
    wip_label.add_css_class("title-1");
    let wip_subtitle = gtk4::Label::new(Some("This feature is under development"));
    wip_subtitle.add_css_class("dim-label"); wip_subtitle.add_css_class("title-4");
    keys_box.append(&wip_icon); keys_box.append(&wip_label); keys_box.append(&wip_subtitle);
    stack.add_named(&keys_box, Some("ssh_keys"));

    let settings_box = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
    settings_box.set_margin_top(48); settings_box.set_margin_bottom(48); settings_box.set_margin_start(48); settings_box.set_margin_end(48);
    settings_box.set_halign(gtk4::Align::Center); settings_box.set_valign(gtk4::Align::Center);
    let settings_icon = gtk4::Image::from_icon_name("emblem-system-symbolic");
    settings_icon.set_pixel_size(96); settings_icon.add_css_class("dim-label");
    let settings_label = gtk4::Label::new(Some("Settings - WIP"));
    settings_label.add_css_class("title-1");
    let settings_subtitle = gtk4::Label::new(Some("This feature is under development"));
    settings_subtitle.add_css_class("dim-label"); settings_subtitle.add_css_class("title-4");
    settings_box.append(&settings_icon); settings_box.append(&settings_label); settings_box.append(&settings_subtitle);
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
