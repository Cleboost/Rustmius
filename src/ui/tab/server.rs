use crate::service::{
    SshServer, drop_onto_folder_into, get_folder_path, get_items_in_folder, get_servers_in_folder,
    layout::LayoutItem, load_layout, load_ssh_servers, remove_server_from_anywhere, save_layout,
};
use crate::ui::component::icon_button::create_icon_button;
use crate::ui::component::{FolderCardConfig, create_folder_card, create_server_card};
use crate::ui::modal::rename::create_rename_dialog;
use gtk4::gdk;
use gtk4::glib::Type;
use gtk4::prelude::*;
use gtk4::{Box, Button, DropTarget, Entry, Label, Orientation, ScrolledWindow};
use libadwaita::StatusPage;
use std::cell::RefCell;
use std::rc::Rc;

fn count_servers_recursive(items: &[LayoutItem]) -> usize {
    let mut count = 0;
    for item in items {
        match item {
            LayoutItem::Server { .. } => count += 1,
            LayoutItem::Folder { items, .. } => count += count_servers_recursive(items),
        }
    }
    count
}

pub fn create_server_tab(
    parent_window: Option<&libadwaita::ApplicationWindow>,
) -> (Box, Rc<dyn Fn() + '_>) {
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

    let current_folder = Rc::new(RefCell::new(None::<String>));

    let breadcrumb_container = Box::new(Orientation::Horizontal, 8);
    breadcrumb_container.set_margin_start(20);
    breadcrumb_container.set_margin_top(10);
    breadcrumb_container.set_margin_bottom(10);
    breadcrumb_container.set_halign(gtk4::Align::Start);

    let back_button = Button::builder()
        .icon_name("go-previous-symbolic")
        .css_classes(vec!["circular", "flat"])
        .tooltip_text("Back to parent folder")
        .build();
    back_button.set_sensitive(false);

    let hosts_button = Button::builder()
        .label("Hosts")
        .css_classes(vec!["flat"])
        .build();
    hosts_button.add_css_class("title-3");

    let hosts_drop_target = DropTarget::new(Type::STRING, gdk::DragAction::MOVE);
    {
        let servers_data = Rc::clone(&servers_data);
        let refresh_fn_storage = Rc::clone(&refresh_fn_storage);
        hosts_drop_target.connect_drop(move |_w, value, _x, _y| {
            if let Ok(server_name) = value.get::<String>() {
                let mut layout = {
                    let servers = servers_data.borrow();
                    load_layout(&servers)
                }; // servers borrow is released here

                remove_server_from_anywhere(&mut layout, &server_name);

                layout.items.push(LayoutItem::Server { name: server_name });

                if let Err(e) = save_layout(&layout) {
                    eprintln!("Failed to save layout: {}", e);
                } else {
                    if let Some(refresh_fn) = refresh_fn_storage.borrow().as_ref() {
                        refresh_fn();
                    }
                    return true;
                }
            }
            false
        });
    }
    hosts_button.add_controller(hosts_drop_target);

    container.append(&breadcrumb_container);

    let update_breadcrumb = {
        let breadcrumb_container = breadcrumb_container.clone();
        let back_button = back_button.clone();
        let hosts_button = hosts_button.clone();
        let servers_data = Rc::clone(&servers_data);
        let current_folder = Rc::clone(&current_folder);
        let refresh_fn_storage = Rc::clone(&refresh_fn_storage);

        Rc::new(move || {
            while let Some(child) = breadcrumb_container.first_child() {
                breadcrumb_container.remove(&child);
            }

            breadcrumb_container.append(&back_button);
            breadcrumb_container.append(&hosts_button);

            if let Some(ref folder_name) = *current_folder.borrow() {
                let servers = servers_data.borrow();
                let layout = load_layout(&servers);
                let path = get_folder_path(&layout, folder_name);

                for (_i, folder_in_path) in path.iter().enumerate() {
                    let separator = Label::new(Some(">"));
                    separator.add_css_class("dim-label");
                    breadcrumb_container.append(&separator);

                    let folder_button = Button::builder()
                        .label(folder_in_path)
                        .css_classes(vec!["flat"])
                        .build();
                    folder_button.add_css_class("title-3");

                    let current_folder_clone = Rc::clone(&current_folder);
                    let folder_name_clone = folder_in_path.clone();
                    let refresh_fn_storage_click = Rc::clone(&refresh_fn_storage);

                    folder_button.connect_clicked(move |_| {
                        *current_folder_clone.borrow_mut() = Some(folder_name_clone.clone());
                        if let Some(refresh_fn) = refresh_fn_storage_click.borrow().as_ref() {
                            refresh_fn();
                        }
                    });

                    let folder_drop_target = DropTarget::new(Type::STRING, gdk::DragAction::MOVE);
                    {
                        let servers_data_clone = Rc::clone(&servers_data);
                        let refresh_fn_storage_clone = Rc::clone(&refresh_fn_storage);
                        let folder_name_for_drop = folder_in_path.clone();
                        folder_drop_target.connect_drop(move |_w, value, _x, _y| {
                            if let Ok(server_name) = value.get::<String>() {
                                let mut layout = {
                                    let servers = servers_data_clone.borrow();
                                    load_layout(&servers)
                                };

                                if let Err(e) = drop_onto_folder_into(
                                    &mut layout,
                                    &server_name,
                                    &folder_name_for_drop,
                                ) {
                                    eprintln!("Failed to move server to folder: {}", e);
                                } else {
                                    if let Err(e) = save_layout(&layout) {
                                        eprintln!("Failed to save layout: {}", e);
                                    } else {
                                        if let Some(refresh_fn) =
                                            refresh_fn_storage_clone.borrow().as_ref()
                                        {
                                            refresh_fn();
                                        }
                                        return true;
                                    }
                                }
                            }
                            false
                        });
                    }
                    folder_button.add_controller(folder_drop_target);

                    breadcrumb_container.append(&folder_button);
                }

                if let Some(ref folder_name) = *current_folder.borrow() {
                    let separator = Label::new(Some(">"));
                    separator.add_css_class("dim-label");
                    breadcrumb_container.append(&separator);

                    let edit_button = Button::builder()
                        .icon_name("document-edit-symbolic")
                        .css_classes(vec!["circular", "flat"])
                        .tooltip_text("Edit folder name")
                        .build();

                    let current_folder_clone = Rc::clone(&current_folder);
                    let servers_data_clone = Rc::clone(&servers_data);
                    let refresh_fn_storage_clone = Rc::clone(&refresh_fn_storage);
                    let folder_name_initial = folder_name.clone();
                    edit_button.connect_clicked(move |_| {
                        let value_servers = servers_data_clone.clone();
                        let value_current = current_folder_clone.clone();
                        let value_key = folder_name_initial.clone();
                        let value_refresh = refresh_fn_storage_clone.clone();
                        let dialog = create_rename_dialog(
                            "Rename Folder",
                            &folder_name_initial,
                            Some(std::boxed::Box::new(move |new_name: String| {
                                {
                                    let servers = value_servers.borrow();
                                    let mut layout = load_layout(&servers);
                                    if let Err(e) = crate::service::rename_folder(
                                        &mut layout,
                                        &value_key,
                                        &new_name,
                                    ) {
                                        eprintln!("Failed to rename folder: {}", e);
                                    } else if let Err(e) = save_layout(&layout) {
                                        eprintln!("Failed to save layout: {}", e);
                                    } else {
                                        *value_current.borrow_mut() = Some(new_name.clone());
                                    }
                                }
                                if let Some(cb) = value_refresh.borrow().as_ref() {
                                    cb();
                                }
                            })),
                            None,
                        );
                        dialog.show();
                    });

                    breadcrumb_container.append(&edit_button);
                }

                back_button.set_sensitive(true);
            } else {
                back_button.set_sensitive(false);
            }
        })
    };

    let create_server_cards = {
        let servers_data = Rc::clone(&servers_data);
        let server_cards = Rc::clone(&server_cards);
        let servers_grid = servers_grid.clone();
        let parent_window_clone = parent_window.cloned();
        let refresh_fn_storage = Rc::clone(&refresh_fn_storage);
        let current_folder = Rc::clone(&current_folder);

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
            let is_filtering = !filter.is_empty();
            let current_folder_name = current_folder.borrow().clone();

            let filtered_servers: Vec<&SshServer> = if let Some(folder_name) = &current_folder_name
            {
                let layout = load_layout(&servers);
                let server_names = get_servers_in_folder(&layout, folder_name);
                let mut folder_servers = Vec::new();

                for server_name in server_names {
                    if let Some(server) = servers.iter().find(|s| s.name == server_name) {
                        folder_servers.push(server);
                    }
                }

                if is_filtering {
                    folder_servers.retain(|server| {
                        server.name.to_lowercase().contains(&filter.to_lowercase())
                            || server.hostname.as_ref().map_or(false, |hostname| {
                                hostname.to_lowercase().contains(&filter.to_lowercase())
                            })
                    });
                }
                folder_servers
            } else if is_filtering {
                servers
                    .iter()
                    .filter(|server| {
                        server.name.to_lowercase().contains(&filter.to_lowercase())
                            || server.hostname.as_ref().map_or(false, |hostname| {
                                hostname.to_lowercase().contains(&filter.to_lowercase())
                            })
                    })
                    .collect()
            } else {
                servers.iter().collect()
            };

            let should_show_empty = if let Some(ref folder_name) = current_folder_name {
                let layout = load_layout(&servers);
                let server_names = get_servers_in_folder(&layout, folder_name);
                server_names.is_empty() && !is_filtering
            } else {
                filtered_servers.is_empty() && (is_filtering || current_folder_name.is_some())
            };

            if should_show_empty {
                servers_grid.set_halign(gtk4::Align::Center);
                servers_grid.set_valign(gtk4::Align::Center);

                let status_page = if let Some(_) = current_folder_name {
                    StatusPage::builder()
                        .icon_name("folder-symbolic")
                        .title("This folder is empty")
                        .build()
                } else if filter.is_empty() {
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

                let refresh_callback = refresh_fn_storage.borrow().as_ref().map(|f| Rc::clone(f));

                if let Some(ref folder_name) = current_folder_name {
                    let layout = load_layout(&servers);
                    let items_to_show = get_items_in_folder(&layout, folder_name);

                    let mut folder_items: Vec<&LayoutItem> = Vec::new();
                    let mut server_items: Vec<&LayoutItem> = Vec::new();
                    for it in items_to_show.iter() {
                        match it {
                            LayoutItem::Folder { .. } => folder_items.push(it),
                            _ => server_items.push(it),
                        }
                    }
                    let ordered: Vec<&LayoutItem> = folder_items
                        .into_iter()
                        .chain(server_items.into_iter())
                        .collect();

                    for (i, item) in ordered.iter().enumerate() {
                        match *item {
                            LayoutItem::Server { name: server_name } => {
                                if let Some(server) =
                                    servers.iter().find(|s| s.name == *server_name)
                                {
                                    if !is_filtering
                                        || server
                                            .name
                                            .to_lowercase()
                                            .contains(&filter.to_lowercase())
                                        || server.hostname.as_ref().map_or(false, |hostname| {
                                            hostname.to_lowercase().contains(&filter.to_lowercase())
                                        })
                                    {
                                        let server_card = create_server_card(
                                            server,
                                            parent_window_clone.as_ref(),
                                            refresh_callback.clone(),
                                            current_folder_name.clone(),
                                        );
                                        server_cards.borrow_mut().push(server_card.clone());
                                        current_row.append(&server_card);
                                    }
                                }
                            }
                            LayoutItem::Folder { id, name, items } => {
                                let subfolder_server_count = count_servers_recursive(items);
                                if subfolder_server_count > 0 {
                                    let current_folder_clone = Rc::clone(&current_folder);
                                    let refresh_callback_clone = refresh_callback.clone();

                                    let subfolder_name = name.clone();
                                    let navigate_to_subfolder =
                                        Rc::new(move |_subfolder_name: &str| {
                                            *current_folder_clone.borrow_mut() =
                                                Some(subfolder_name.clone());
                                            if let Some(refresh_fn) = &refresh_callback_clone {
                                                refresh_fn();
                                            }
                                        });

                                    let subfolder_card = create_folder_card(
                                        FolderCardConfig {
                                            folder_id: id,
                                            folder_name: name,
                                        },
                                        parent_window_clone.as_ref(),
                                        refresh_callback.clone(),
                                        Some(navigate_to_subfolder),
                                    );
                                    current_row.append(&subfolder_card);
                                }
                            }
                        }

                        if (i + 1) % cards_per_row == 0 || i == ordered.len() - 1 {
                            servers_grid.append(&current_row);
                            if i < ordered.len() - 1 {
                                current_row = Box::new(Orientation::Horizontal, 12);
                                current_row.set_halign(gtk4::Align::Start);
                                current_row.set_hexpand(true);
                            }
                        }
                    }
                } else if is_filtering {
                    for (i, server) in filtered_servers.iter().enumerate() {
                        let server_card = create_server_card(
                            server,
                            parent_window_clone.as_ref(),
                            refresh_callback.clone(),
                            current_folder_name.clone(),
                        );
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
                } else {
                    let layout = load_layout(&servers);
                    let mut by_name: std::collections::HashMap<&str, &SshServer> =
                        std::collections::HashMap::new();
                    for s in servers.iter() {
                        by_name.insert(&s.name, s);
                    }

                    let mut folder_items: Vec<&LayoutItem> = Vec::new();
                    let mut server_items: Vec<&LayoutItem> = Vec::new();
                    for it in layout.items.iter() {
                        match it {
                            LayoutItem::Folder { .. } => folder_items.push(it),
                            _ => server_items.push(it),
                        }
                    }
                    let ordered: Vec<&LayoutItem> = folder_items
                        .into_iter()
                        .chain(server_items.into_iter())
                        .collect();

                    let mut index_in_row = 0usize;
                    for item in ordered.iter() {
                        match *item {
                            crate::service::layout::LayoutItem::Server { name } => {
                                if let Some(server) = by_name.get::<str>(&name) {
                                    let server_card = create_server_card(
                                        server,
                                        parent_window_clone.as_ref(),
                                        refresh_callback.clone(),
                                        current_folder_name.clone(),
                                    );
                                    server_cards.borrow_mut().push(server_card.clone());
                                    current_row.append(&server_card);
                                    index_in_row += 1;
                                }
                            }
                            crate::service::layout::LayoutItem::Folder { id, name, items } => {
                                let server_count = count_servers_recursive(items);
                                if server_count > 0 {
                                    let folder_name = name.to_string();
                                    let current_folder_clone = Rc::clone(&current_folder);
                                    let refresh_callback_clone = refresh_callback.clone();

                                    let navigate_to_folder = Rc::new(move |_folder_name: &str| {
                                        *current_folder_clone.borrow_mut() =
                                            Some(folder_name.clone());
                                        if let Some(refresh_fn) = &refresh_callback_clone {
                                            refresh_fn();
                                        }
                                    });

                                    let folder_card = create_folder_card(
                                        FolderCardConfig {
                                            folder_id: id,
                                            folder_name: name,
                                        },
                                        parent_window_clone.as_ref(),
                                        refresh_callback.clone(),
                                        Some(navigate_to_folder),
                                    );
                                    current_row.append(&folder_card);
                                    index_in_row += 1;
                                }
                            }
                        }

                        if index_in_row % cards_per_row == 0 {
                            servers_grid.append(&current_row);
                            current_row = Box::new(Orientation::Horizontal, 12);
                            current_row.set_halign(gtk4::Align::Start);
                            current_row.set_hexpand(true);
                        }
                    }

                    if index_in_row % cards_per_row != 0 || index_in_row == 0 {
                        servers_grid.append(&current_row);
                    }
                }
            }
        })
    };

    let navigate_to_hosts = {
        let current_folder = Rc::clone(&current_folder);
        let create_server_cards = Rc::clone(&create_server_cards);
        let search_entry = search_entry.clone();
        let update_breadcrumb = Rc::clone(&update_breadcrumb);
        Rc::new(move || {
            *current_folder.borrow_mut() = None;
            let text = search_entry.text();
            create_server_cards(&text, None);
            update_breadcrumb();
        })
    };

    let navigate_back = {
        let current_folder = Rc::clone(&current_folder);
        let create_server_cards = Rc::clone(&create_server_cards);
        let search_entry = search_entry.clone();
        let update_breadcrumb = Rc::clone(&update_breadcrumb);
        Rc::new(move || {
            *current_folder.borrow_mut() = None;
            let text = search_entry.text();
            create_server_cards(&text, None);
            update_breadcrumb();
        })
    };

    let navigate_to_hosts_clone = Rc::clone(&navigate_to_hosts);
    hosts_button.connect_clicked(move |_| {
        navigate_to_hosts_clone();
    });

    let navigate_back_clone = Rc::clone(&navigate_back);
    back_button.connect_clicked(move |_| {
        navigate_back_clone();
    });

    let refresh_fn: Rc<dyn Fn()> = Rc::new({
        let search_entry = search_entry.clone();
        let create_server_cards = Rc::clone(&create_server_cards);
        let update_breadcrumb = Rc::clone(&update_breadcrumb);
        move || {
            let text = search_entry.text();
            create_server_cards(&text, None);
            update_breadcrumb();
        }
    });

    *refresh_fn_storage.borrow_mut() = Some(Rc::clone(&refresh_fn));
    create_server_cards("", Some(&refresh_fn));
    update_breadcrumb();

    let create_server_cards_clone = Rc::clone(&create_server_cards);
    search_entry.connect_changed(move |entry| {
        let text = entry.text();
        create_server_cards_clone(&text, None);
    });

    scrolled.set_child(Some(&servers_grid));
    container.append(&scrolled);

    (container, refresh_fn)
}
