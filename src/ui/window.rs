use gtk4::prelude::*;
use gtk4::{glib, gio};
use crate::ui::server_list::{ServerList, ServerAction};
use crate::ui::add_server_dialog::show_server_dialog;
use crate::config_observer::{add_host_to_config, delete_host_from_config, SshHost, load_hosts};
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

    // Sidebar
    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    sidebar.set_width_request(60);
    sidebar.set_margin_top(12);
    let btn_servers = gtk4::Button::from_icon_name("network-server-symbolic");
    btn_servers.add_css_class("flat");
    let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    spacer.set_vexpand(true);
    sidebar.append(&btn_servers);
    sidebar.append(&spacer);

    let separator = gtk4::Separator::new(gtk4::Orientation::Vertical);

    // Content area
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

    // Terminal
    let terminal_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    let terminal = vte4::Terminal::new();
    terminal.set_vexpand(true);
    terminal_box.append(&terminal);

    // Refresh UI logic
    let refresh_ui: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    
    let stack_clone = stack.clone();
    let terminal_clone = terminal.clone();
    let window_clone = window.clone();
    let refresh_ui_weak = Rc::downgrade(&refresh_ui);

    let do_refresh = Rc::new(move || {
        let stack = stack_clone.clone();
        let terminal = terminal_clone.clone();
        let window = window_clone.clone();
        let refresh_ui_handle = refresh_ui_weak.upgrade().unwrap();
        
        if let Some(child) = stack.child_by_name("server_grid") {
            stack.remove(&child);
        }

        let sl_stack = stack.clone();
        let sl_terminal = terminal.clone();
        let sl_window = window.clone();
        let sl_refresh_handle = refresh_ui_handle.clone();

        let sl = ServerList::new(move |action| {
            let stack = sl_stack.clone();
            let terminal = sl_terminal.clone();
            let window = sl_window.clone();
            let refresh = sl_refresh_handle.borrow().as_ref().unwrap().clone();
            
            match action {
                ServerAction::Connect(host) => {
                    stack.set_visible_child_name("terminal");
                    let exe_path = std::env::current_exe().unwrap_or_default().to_string_lossy().to_string();
                    let mut envv: Vec<String> = std::env::vars().map(|(k, v)| format!("{}={}", k, v)).collect();
                    envv.push(format!("SSH_ASKPASS={}", exe_path));
                    envv.push("SSH_ASKPASS_REQUIRE=force".to_string());
                    envv.push(format!("RUSTMIUS_ASKPASS_ALIAS={}", host.alias));
                    envv.push("DISPLAY=:0".to_string());
                    let env_refs: Vec<&str> = envv.iter().map(|s| s.as_str()).collect();
                    let user_str = host.user.unwrap_or_else(|| "root".to_string());
                    terminal.spawn_async(vte4::PtyFlags::DEFAULT, None, &["/usr/bin/ssh", "-o", "StrictHostKeyChecking=no", "-o", "PubkeyAuthentication=no", &format!("{}@{}", user_str, host.hostname)], &env_refs, glib::SpawnFlags::SEARCH_PATH, || {}, -1, None::<&gio::Cancellable>, |_| {});
                },
                ServerAction::Delete(host) => {
                    let _ = delete_host_from_config(&host.alias);
                    let alias_norm = host.alias.to_lowercase();
                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(keyring) = oo7::Keyring::new().await {
                            let mut attr = std::collections::HashMap::new();
                            attr.insert("rustmius-server-alias", alias_norm);
                            if let Ok(items) = keyring.search_items(&attr).await {
                                for item in items { let _ = item.delete().await; }
                            }
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
                                        attr.insert("rustmius-server-alias", alias_lower);
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
        stack.add_named(&sl.container, Some("server_grid"));
        stack.set_visible_child_name("server_grid");
    });

    *refresh_ui.borrow_mut() = Some(do_refresh.clone());

    // Initial load
    do_refresh();

    // Add button
    let refresh_add = do_refresh.clone();
    let window_add = window.clone();
    add_btn.connect_clicked(move |_| {
        let refresh = refresh_add.clone();
        let existing_aliases: Vec<String> = load_hosts().into_iter().map(|h| h.alias.to_lowercase()).collect();
        
        show_server_dialog(window_add.upcast_ref(), None, existing_aliases, move |new_host, password| {
            if let Ok(_) = add_host_to_config(&new_host) {
                if !password.is_empty() {
                    let host_alias = new_host.alias.clone();
                    glib::MainContext::default().spawn_local(async move {
                        if let Ok(keyring) = oo7::Keyring::new().await {
                            let mut attr = std::collections::HashMap::new();
                            let alias_lower = host_alias.to_lowercase();
                            attr.insert("rustmius-server-alias", alias_lower);
                            let _ = keyring.create_item(&format!("Rustmius: SSH Password for {}", host_alias), &attr, password.as_bytes(), true).await;
                        }
                    });
                }
                refresh();
            }
        });
    });

    // Navigation
    let stack_nav = stack.clone();
    btn_servers.connect_clicked(move |_| { stack_nav.set_visible_child_name("server_grid"); });

    stack.add_named(&terminal_box, Some("terminal"));
    root.append(&sidebar);
    root.append(&separator);
    root.append(&content_box);
    window.set_child(Some(&root));
    window.present();
}
