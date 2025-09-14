use crate::service::SshServer;
use crate::service::{append_history, load_settings};
use crate::ui::bus::emit_history_refresh;
use crate::ui::modal::{
    delete_server::create_delete_server_dialog, edit_server::create_edit_server_dialog,
};
use gtk4::gdk;
use gtk4::glib::{Type, Value};
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, DragSource, Frame, Image, Label, Orientation};
use libadwaita::prelude::AdwDialogExt;
use std::process::Command;
use std::rc::Rc;

pub fn create_server_card(
    server: &SshServer,
    parent_window: Option<&libadwaita::ApplicationWindow>,
    on_save: Option<std::rc::Rc<dyn Fn() + 'static>>,
    current_folder: Option<String>,
) -> Frame {
    let card = Frame::new(None);
    card.add_css_class("server-card");
    card.add_css_class("hoverable");
    card.set_can_focus(true);
    card.set_focus_on_click(true);
    card.set_cursor_from_name(Some("pointer"));
    card.set_margin_start(8);
    card.set_margin_end(8);
    card.set_margin_top(8);
    card.set_margin_bottom(8);
    card.set_width_request(300);

    let main_container = GtkBox::new(Orientation::Vertical, 12);
    main_container.set_margin_top(16);
    main_container.set_margin_bottom(16);
    main_container.set_margin_start(16);
    main_container.set_margin_end(16);

    let header_container = GtkBox::new(Orientation::Horizontal, 12);
    header_container.set_halign(gtk4::Align::Start);

    let server_icon = Image::from_icon_name("network-server-symbolic");
    server_icon.set_icon_size(gtk4::IconSize::Large);
    header_container.append(&server_icon);

    let title_container = GtkBox::new(Orientation::Vertical, 4);
    title_container.set_halign(gtk4::Align::Start);
    title_container.set_hexpand(true);
    title_container.set_margin_end(16);

    let server_name = Label::new(Some(&server.name));
    server_name.add_css_class("title-3");
    server_name.set_halign(gtk4::Align::Start);
    title_container.append(&server_name);

    let hostname_text = server.hostname.as_deref().unwrap_or("No hostname");
    let hostname_label = Label::new(Some(hostname_text));
    hostname_label.add_css_class("dim-label");
    hostname_label.set_halign(gtk4::Align::Start);
    title_container.append(&hostname_label);

    header_container.append(&title_container);

    let is_special_host = server.name == "aur.archlinux.org";

    let actions_container = GtkBox::new(Orientation::Horizontal, 8);
    actions_container.set_halign(gtk4::Align::End);
    actions_container.set_valign(gtk4::Align::Center);

    let connect_button = Button::builder()
        .label("Connect")
        .css_classes(vec!["suggested-action"])
        .build();

    if is_special_host {
        connect_button.set_sensitive(false);
    } else {
        connect_button.set_tooltip_text(Some("Connect to this server"));
    }
    actions_container.append(&connect_button);

    let edit_button = Button::builder()
        .icon_name("edit-symbolic")
        .css_classes(vec!["circular", "flat"])
        .build();

    if is_special_host {
        edit_button.set_sensitive(false);
        edit_button.set_tooltip_text(Some("This host is special and can't be edited"));
    } else {
        edit_button.set_tooltip_text(Some("Edit server configuration"));
    }
    actions_container.append(&edit_button);

    let delete_button = Button::builder()
        .icon_name("user-trash-symbolic")
        .css_classes(vec!["circular", "flat", "destructive-action"])
        .build();

    if is_special_host {
        delete_button.set_sensitive(false);
        delete_button.set_tooltip_text(Some("This host is special and can't be deleted"));
    } else {
        delete_button.set_tooltip_text(Some("Delete server"));
    }
    actions_container.append(&delete_button);

    header_container.append(&actions_container);
    main_container.append(&header_container);

    if let Some(port) = server.port {
        let details_container = GtkBox::new(Orientation::Vertical, 6);
        details_container.set_margin_top(8);

        let port_label = Label::new(Some(&format!("Port: {}", port)));
        port_label.add_css_class("caption");
        port_label.set_halign(gtk4::Align::Start);
        details_container.append(&port_label);

        main_container.append(&details_container);
    }

    let server_name_clone = server.name.clone();
    connect_button.connect_clicked(move |_| {
        // Record history once per click if enabled
        let settings = load_settings();
        if settings.remember_servers {
            if let Err(e) = append_history(&server_name_clone) {
                eprintln!("Failed to append history: {}", e);
            } else {
                emit_history_refresh();
            }
        }

        let terminal_commands = vec![
            ("foot", vec!["-e", "ssh", &server_name_clone]),
            ("gnome-terminal", vec!["--", "ssh", &server_name_clone]),
            ("konsole", vec!["-e", "ssh", &server_name_clone]),
            ("xterm", vec!["-e", "ssh", &server_name_clone]),
            ("alacritty", vec!["-e", "ssh", &server_name_clone]),
            ("kitty", vec!["ssh", &server_name_clone]),
            ("terminator", vec!["-e", "ssh", &server_name_clone]),
            ("xfce4-terminal", vec!["-e", "ssh", &server_name_clone]),
            ("mate-terminal", vec!["-e", "ssh", &server_name_clone]),
            ("lxterminal", vec!["-e", "ssh", &server_name_clone]),
        ];

        let mut success = false;
        for (terminal, args) in terminal_commands {
            let result = Command::new(terminal).args(&args).spawn();

            match result {
                Ok(_) => {
                    success = true;
                    break;
                }
                Err(_) => {
                    continue;
                }
            }
        }

        if !success {
            let fallback_result = Command::new("sh")
                .arg("-c")
                .arg(&format!("${{TERM:-xterm}} -e ssh {}", server_name_clone))
                .spawn();

            match fallback_result {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("Failed to open any terminal for SSH connection: {}", e);
                }
            }
        }
    });

    let server_clone = server.clone();
    let parent_window_clone = parent_window.cloned();
    let on_save_clone = on_save.clone();
    edit_button.connect_clicked(move |_| {
        let edit_dialog = create_edit_server_dialog(
            &server_clone,
            on_save_clone.as_ref().map(|f| {
                let f_clone = Rc::clone(f);
                std::boxed::Box::new(move || f_clone()) as std::boxed::Box<dyn Fn() + 'static>
            }),
        );
        if let Some(parent) = &parent_window_clone {
            edit_dialog.present(Some(parent));
        } else {
            edit_dialog.show();
        }
    });

    let server_clone_for_delete = server.clone();
    let parent_window_clone_for_delete = parent_window.cloned();
    let on_save_clone_for_delete = on_save.clone();
    delete_button.connect_clicked(move |_| {
        let delete_dialog = create_delete_server_dialog(
            &server_clone_for_delete.name,
            on_save_clone_for_delete.clone(),
        );
        if let Some(parent) = &parent_window_clone_for_delete {
            delete_dialog.present(Some(parent));
        } else {
            delete_dialog.show();
        }
    });

    card.set_child(Some(&main_container));

    if is_special_host {
        card.set_tooltip_text(Some("This host is special host and can't be edit"));
    }

    let drag_source = DragSource::builder()
        .actions(gdk::DragAction::COPY.union(gdk::DragAction::MOVE))
        .build();
    {
        let name = server.name.clone();
        drag_source.connect_prepare(move |_w, _x, _y| {
            let value = Value::from(name.clone());
            Some(gtk4::gdk::ContentProvider::for_value(&value))
        });
    }
    card.add_controller(drag_source);

    let drop_target = gtk4::DropTarget::new(
        Type::STRING,
        gdk::DragAction::COPY.union(gdk::DragAction::MOVE),
    );
    {
        let target_name = server.name.clone();
        let on_save = on_save.clone();
        let current_folder_context = current_folder.clone();
        drop_target.connect_drop(move |_w, value, _x, _y| {
            if let Ok(source_name) = value.get::<String>() {
                if source_name == target_name {
                    return false;
                }
                let servers = match crate::service::load_ssh_servers() {
                    Ok(v) => v,
                    Err(_) => vec![],
                };
                let mut layout = crate::service::load_layout(&servers);

                if let Some(folder_name) = &current_folder_context {
                    match crate::service::drop_onto_server_into_folder(
                        &mut layout,
                        &source_name,
                        &target_name,
                        folder_name,
                    ) {
                        Ok(_) => {
                            let _ = crate::service::save_layout(&layout);
                            if let Some(cb) = &on_save {
                                cb();
                            }
                            return true;
                        }
                        Err(e) => {
                            eprintln!("Drop onto server in folder failed: {}", e);
                        }
                    }
                } else {
                    match crate::service::drop_onto_server_into(
                        &mut layout,
                        &source_name,
                        &target_name,
                    ) {
                        Ok(_) => {
                            let _ = crate::service::save_layout(&layout);
                            if let Some(cb) = &on_save {
                                cb();
                            }
                            return true;
                        }
                        Err(e) => {
                            eprintln!("Drop onto server failed: {}", e);
                        }
                    }
                }
            }
            false
        });
    }
    card.add_controller(drop_target);

    card
}
