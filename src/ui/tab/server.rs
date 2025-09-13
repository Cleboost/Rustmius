use crate::service::{SshServer, load_ssh_servers};
use crate::ui::component::create_server_card;
use crate::ui::component::icon_button::create_icon_button;
use gtk4::prelude::*;
use gtk4::{Box, Entry, Label, Orientation, ScrolledWindow};
use libadwaita::StatusPage;
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_server_tab(
    parent_window: Option<&libadwaita::ApplicationWindow>,
) -> (Box, Rc<dyn Fn()>) {
    let container = Box::new(Orientation::Vertical, 0);

    let header_container = Box::new(Orientation::Vertical, 20);
    header_container.set_margin_top(20);
    header_container.set_margin_bottom(20);
    header_container.set_margin_start(12);
    header_container.set_margin_end(12);
    header_container.set_halign(gtk4::Align::Fill);
    header_container.set_hexpand(true);

    let search_entry = Entry::builder()
        .placeholder_text("Find a host or ssh user@hostname...")
        .primary_icon_name("search-symbolic")
        .hexpand(true)
        .height_request(35)
        .build();

    search_entry.add_css_class("search-entry");
    header_container.append(&search_entry);

    let buttons_container = Box::new(Orientation::Horizontal, 12);
    buttons_container.set_halign(gtk4::Align::Start);
    buttons_container.set_hexpand(true);

    let refresh_fn_storage = Rc::new(RefCell::new(None::<Rc<dyn Fn()>>));

    let new_host_button = {
        let parent_window_clone = parent_window.cloned();
        let refresh_fn_storage_btn = Rc::clone(&refresh_fn_storage);
        create_icon_button("New Host", "network-server-symbolic", 100, 30, move || {
            if let Some(window) = &parent_window_clone {
                let cb_storage = Rc::clone(&refresh_fn_storage_btn);
                let add_server_dialog = crate::ui::modal::add_server::create_add_server_dialog(
                    Some(std::boxed::Box::new(move || {
                        if let Some(cb) = &*cb_storage.borrow() {
                            cb();
                        }
                    })),
                );
                add_server_dialog.set_transient_for(Some(window));
                add_server_dialog.show();
            }
        })
    };

    let terminal_button =
        create_icon_button("Terminal", "utilities-terminal-symbolic", 100, 30, || {});
    terminal_button.set_sensitive(false);

    let serial_button = create_icon_button("Serial", "network-wired-symbolic", 100, 30, || {});
    serial_button.set_sensitive(false);

    buttons_container.append(&new_host_button);
    buttons_container.append(&terminal_button);
    buttons_container.append(&serial_button);

    header_container.append(&buttons_container);
    container.append(&header_container);

    let servers_title = Label::new(Some("Servers"));
    servers_title.add_css_class("title-2");
    servers_title.set_margin_top(20);
    servers_title.set_margin_start(20);
    servers_title.set_halign(gtk4::Align::Start);
    container.append(&servers_title);

    let scrolled = ScrolledWindow::new();
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    scrolled.set_margin_start(20);
    scrolled.set_margin_end(20);
    scrolled.set_margin_top(20);
    scrolled.set_margin_bottom(20);

    let servers_grid = Box::new(Orientation::Vertical, 12);
    servers_grid.set_halign(gtk4::Align::Start);
    servers_grid.set_hexpand(true);

    let all_servers = match load_ssh_servers() {
        Ok(servers) => servers,
        Err(e) => {
            eprintln!("Error loading SSH servers: {}", e);
            vec![]
        }
    };

    let servers_data = Rc::new(RefCell::new(all_servers));
    let server_cards = Rc::new(RefCell::new(Vec::new()));

    let create_server_cards = {
        let servers_data = Rc::clone(&servers_data);
        let server_cards = Rc::clone(&server_cards);
        let servers_grid = servers_grid.clone();
        let parent_window_clone = parent_window.cloned();
        let refresh_fn_storage = Rc::clone(&refresh_fn_storage);
        Rc::new(move |filter: &str, _refresh_fn: Option<&Rc<dyn Fn()>>| {
            while let Some(child) = servers_grid.first_child() {
                servers_grid.remove(&child);
            }
            server_cards.borrow_mut().clear();

            let updated_servers = match crate::service::load_ssh_servers() {
                Ok(servers) => servers,
                Err(e) => {
                    eprintln!("Erreur lors du rechargement des serveurs: {}", e);
                    servers_data.borrow().clone()
                }
            };
            *servers_data.borrow_mut() = updated_servers;

            let servers = servers_data.borrow();
            let filtered_servers: Vec<&SshServer> = if filter.is_empty() {
                servers.iter().collect()
            } else {
                servers
                    .iter()
                    .filter(|server| {
                        server.name.to_lowercase().contains(&filter.to_lowercase())
                            || server.hostname.as_ref().map_or(false, |hostname| {
                                hostname.to_lowercase().contains(&filter.to_lowercase())
                            })
                    })
                    .collect()
            };

            if filtered_servers.is_empty() {
                servers_grid.set_halign(gtk4::Align::Center);
                servers_grid.set_valign(gtk4::Align::Center);

                let status_page = if filter.is_empty() {
                    StatusPage::builder()
                        .icon_name("network-server-symbolic")
                        .title("No SSH servers found - Add servers to your config file")
                        .build()
                } else {
                    StatusPage::builder()
                        .icon_name("system-search-symbolic")
                        .title(&format!("No servers found matching '{}'", filter))
                        .build()
                };

                servers_grid.append(&status_page);
            } else {
                servers_grid.set_halign(gtk4::Align::Start);
                servers_grid.set_valign(gtk4::Align::Start);
                let cards_per_row = 3;
                let mut current_row = Box::new(Orientation::Horizontal, 12);
                current_row.set_halign(gtk4::Align::Start);
                current_row.set_hexpand(true);

                for (i, server) in filtered_servers.iter().enumerate() {
                    let refresh_callback =
                        refresh_fn_storage.borrow().as_ref().map(|f| Rc::clone(f));
                    let server_card =
                        create_server_card(server, parent_window_clone.as_ref(), refresh_callback);
                    server_cards.borrow_mut().push(server_card.clone());
                    current_row.append(&server_card);

                    if (i + 1) % cards_per_row == 0 || i == filtered_servers.len() - 1 {
                        servers_grid.append(&current_row);
                        if i < filtered_servers.len() - 1 {
                            current_row = Box::new(Orientation::Horizontal, 12);
                            current_row.set_halign(gtk4::Align::Start);
                            current_row.set_hexpand(true);
                        }
                    }
                }
            }
        })
    };

    let refresh_fn: Rc<dyn Fn()> = Rc::new({
        let search_entry = search_entry.clone();
        let create_server_cards = Rc::clone(&create_server_cards);
        move || {
            let text = search_entry.text();
            create_server_cards(&text, None);
        }
    });

    *refresh_fn_storage.borrow_mut() = Some(Rc::clone(&refresh_fn));
    create_server_cards("", Some(&refresh_fn));

    let create_server_cards_clone = Rc::clone(&create_server_cards);
    search_entry.connect_changed(move |entry| {
        let text = entry.text();
        create_server_cards_clone(&text, None);
    });

    scrolled.set_child(Some(&servers_grid));
    container.append(&scrolled);

    (container, refresh_fn)
}
