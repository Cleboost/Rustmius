use gtk4::gdk;
use gtk4::glib::Type;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Frame, Image, Label, Orientation};
use std::rc::Rc;

pub struct FolderCardConfig<'a> {
    pub folder_id: &'a str,
    pub folder_name: &'a str
}

pub fn create_folder_card(
    cfg: FolderCardConfig,
    _parent_window: Option<&libadwaita::ApplicationWindow>,
    on_changed: Option<Rc<dyn Fn() + 'static>>,
    on_folder_click: Option<Rc<dyn Fn(&str) + 'static>>,
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

    let main_container = GtkBox::new(Orientation::Horizontal, 12);
    main_container.set_margin_top(16);
    main_container.set_margin_bottom(16);
    main_container.set_margin_start(16);
    main_container.set_margin_end(16);

    let folder_icon = Image::from_icon_name("folder-symbolic");
    folder_icon.set_icon_size(gtk4::IconSize::Large);
    folder_icon.set_halign(gtk4::Align::Start);
    folder_icon.set_valign(gtk4::Align::Center);

    main_container.append(&folder_icon);

    let server_count = {
        let servers = match crate::service::load_ssh_servers() {
            Ok(v) => v,
            Err(_) => vec![],
        };
        let layout = crate::service::load_layout(&servers);
        crate::service::get_servers_in_folder(&layout, cfg.folder_name).len()
    };

    let info_label = Label::new(Some(&format!(
        "{}\n({} serveurs)",
        cfg.folder_name, server_count
    )));
    info_label.add_css_class("title-4");
    info_label.set_halign(gtk4::Align::Start);
    info_label.set_valign(gtk4::Align::Center);
    info_label.set_hexpand(true);
    main_container.append(&info_label);

    let drop_target = gtk4::DropTarget::new(
        Type::STRING,
        gdk::DragAction::COPY.union(gdk::DragAction::MOVE),
    );
    {
        let folder_id = cfg.folder_id.to_string();
        let on_changed = on_changed.clone();
        drop_target.connect_drop(move |_w, value, _x, _y| {
            if let Ok(name) = value.get::<String>() {
                let servers = match crate::service::load_ssh_servers() {
                    Ok(v) => v,
                    Err(_) => vec![],
                };
                let mut layout = crate::service::load_layout(&servers);

                let is_server = crate::service::server_exists_anywhere(&layout, &name);

                let result = if is_server {
                    crate::service::drop_onto_folder_into(&mut layout, &name, &folder_id)
                } else {
                    crate::service::drop_folder_onto_folder(&mut layout, &name, &folder_id)
                };

                match result {
                    Ok(_) => {
                        let _ = crate::service::save_layout(&layout);
                        if let Some(cb) = &on_changed {
                            cb();
                        }
                        return true;
                    }
                    Err(e) => {
                        eprintln!("Drop into folder failed: {}", e);
                    }
                }
            }
            false
        });
    }
    card.add_controller(drop_target);

    if let Some(on_folder_click) = on_folder_click {
        let folder_name = cfg.folder_name.to_string();
        let gesture = gtk4::GestureClick::new();
        gesture.connect_released(move |_gesture, _n_press, _x, _y| {
            on_folder_click(&folder_name);
        });
        card.add_controller(gesture);
    }

    card.set_child(Some(&main_container));
    card
}
