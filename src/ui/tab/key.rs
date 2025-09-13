use crate::service::load_ssh_keys;
use crate::ui::component::icon_button::create_icon_button;
use crate::ui::component::ssh_key_card::create_ssh_key_card;
use crate::ui::modal::generate_key::create_generate_key_dialog;
use gtk4::prelude::*;
use gtk4::{Box, Label, Orientation, ScrolledWindow};
use libadwaita::ToastOverlay;
use std::cell::RefCell;
use std::rc::Rc;

fn create_keys_content(
    refresh_callback: Rc<dyn Fn()>, 
    toast_overlay: Rc<ToastOverlay>,
    parent_window: Option<&libadwaita::ApplicationWindow>,
) -> Box {
    let container = Box::new(Orientation::Vertical, 0);

    let header_container = Box::new(Orientation::Vertical, 20);
    header_container.set_margin_top(20);
    header_container.set_margin_bottom(20);
    header_container.set_margin_start(12);
    header_container.set_margin_end(12);
    header_container.set_halign(gtk4::Align::Fill);
    header_container.set_hexpand(true);

    let title = Label::new(Some("SSH Keys Management"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);
    header_container.append(&title);

    let description = Label::new(Some("Manage your SSH keys public and private"));
    description.add_css_class("dim-label");
    description.set_halign(gtk4::Align::Start);
    header_container.append(&description);

    let buttons_container = Box::new(Orientation::Horizontal, 12);
    buttons_container.set_halign(gtk4::Align::Start);
    buttons_container.set_hexpand(true);

    let generate_button = create_icon_button(
        "Generate a new SSH key",
        "key-symbolic",
        120,
        30,
        {
            let refresh_callback = Rc::clone(&refresh_callback);
            let toast_overlay = Rc::clone(&toast_overlay);
            let parent_window = parent_window.cloned();
            move || {
                let dialog = create_generate_key_dialog(
                    Rc::clone(&refresh_callback), 
                    Rc::clone(&toast_overlay),
                    parent_window.as_ref().map(|w| w.as_ref()),
                );
                dialog.present();
            }
        },
    );

    let import_button = create_icon_button(
        "Import a SSH key",
        "document-open-symbolic",
        120,
        30,
        || println!("Import a SSH key"),
    );
    import_button.set_sensitive(false);

    buttons_container.append(&generate_button);
    buttons_container.append(&import_button);
    header_container.append(&buttons_container);
    container.append(&header_container);

    let keys_title = Label::new(Some("SSH Keys"));
    keys_title.add_css_class("title-2");
    keys_title.set_margin_top(20);
    keys_title.set_margin_start(20);
    keys_title.set_halign(gtk4::Align::Start);
    container.append(&keys_title);

    let scrolled = ScrolledWindow::new();
    scrolled.set_hexpand(true);
    scrolled.set_vexpand(true);
    scrolled.set_margin_start(20);
    scrolled.set_margin_end(20);
    scrolled.set_margin_top(20);
    scrolled.set_margin_bottom(20);

    let keys_grid = Box::new(Orientation::Vertical, 12);
    keys_grid.set_halign(gtk4::Align::Start);
    keys_grid.set_hexpand(true);

    let keys_data = match load_ssh_keys() {
        Ok(keys) => keys
            .into_iter()
            .map(|key| {
                (
                    key.name,
                    key.key_type,
                    key.fingerprint,
                    key.has_public,
                    key.has_private,
                    key.file_path,
                )
            })
            .collect::<Vec<_>>(),
        Err(e) => {
            eprintln!("Error loading SSH keys: {}", e);
            vec![]
        }
    };

    if keys_data.is_empty() {
        let no_keys_label = Label::new(Some(
            "No SSH keys found in ~/.ssh/\n\nGenerate a new key or import an existing key.",
        ));
        no_keys_label.add_css_class("dim-label");
        no_keys_label.set_halign(gtk4::Align::Center);
        no_keys_label.set_valign(gtk4::Align::Center);
        no_keys_label.set_hexpand(true);
        no_keys_label.set_vexpand(true);
        keys_grid.append(&no_keys_label);
    } else {
        let cards_per_row = 3;
        let mut current_row = Box::new(Orientation::Horizontal, 12);
        current_row.set_halign(gtk4::Align::Start);
        current_row.set_hexpand(true);

        for (i, (name, key_type, fingerprint, has_public, has_private, file_path)) in
            keys_data.iter().enumerate()
        {
            let ssh_key_card = create_ssh_key_card(
                name,
                key_type,
                fingerprint,
                *has_public,
                *has_private,
                file_path,
                Rc::clone(&refresh_callback),
                Rc::clone(&toast_overlay),
            );
            current_row.append(&ssh_key_card);

            if (i + 1) % cards_per_row == 0 || i == keys_data.len() - 1 {
                keys_grid.append(&current_row);
                if i < keys_data.len() - 1 {
                    current_row = Box::new(Orientation::Horizontal, 12);
                    current_row.set_halign(gtk4::Align::Start);
                    current_row.set_hexpand(true);
                }
            }
        }
    }

    scrolled.set_child(Some(&keys_grid));
    container.append(&scrolled);

    container
}

pub fn create_key_tab(
    toast_overlay: Rc<ToastOverlay>,
    parent_window: Option<&libadwaita::ApplicationWindow>,
) -> (Box, Rc<dyn Fn()>) {
    let main_container = Box::new(Orientation::Vertical, 0);
    let refresh_fn_cell = Rc::new(RefCell::new(None::<Rc<dyn Fn()>>));

    let parent_window_clone = parent_window.cloned();
    let refresh_fn: Rc<dyn Fn()> = Rc::new({
        let main_container = main_container.clone();
        let refresh_fn_cell = Rc::clone(&refresh_fn_cell);
        let toast_overlay = Rc::clone(&toast_overlay);
        let parent_window = parent_window_clone.clone();
        move || {
            if let Some(child) = main_container.first_child() {
                main_container.remove(&child);
            }

            let new_keys_content = create_keys_content(
                Rc::clone(&refresh_fn_cell.borrow().as_ref().unwrap()),
                Rc::clone(&toast_overlay),
                parent_window.as_ref().map(|w| w.as_ref()),
            );
            main_container.append(&new_keys_content);
        }
    });

    *refresh_fn_cell.borrow_mut() = Some(Rc::clone(&refresh_fn));

    let keys_content = create_keys_content(Rc::clone(&refresh_fn), Rc::clone(&toast_overlay), parent_window);
    main_container.append(&keys_content);

    (main_container, refresh_fn)
}
