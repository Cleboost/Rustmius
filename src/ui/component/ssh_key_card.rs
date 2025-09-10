use crate::service::{delete_key_pair, read_key_content, regenerate_public_key};
use crate::ui::modal::delete_key::create_delete_key_dialog;
use crate::ui::modal::info_key::create_info_key_dialog;
use gtk4::prelude::*;
use gtk4::{Box, Button, Frame, GestureClick, Image, Label, Orientation};
use libadwaita::ToastOverlay;
use libadwaita::prelude::*;
use std::rc::Rc;

pub fn create_ssh_key_card(
    name: &str,
    key_type: &str,
    _fingerprint: &str,
    has_public: bool,
    has_private: bool,
    private_key_path: &str,
    refresh_callback: Rc<dyn Fn()>,
    toast_overlay: Rc<ToastOverlay>,
) -> Frame {
    let card = Frame::new(None);
    card.add_css_class("ssh-key-card");
    card.add_css_class("hoverable");
    card.set_can_focus(true);
    card.set_focus_on_click(true);
    card.set_cursor_from_name(Some("pointer"));
    card.set_margin_start(8);
    card.set_margin_end(8);
    card.set_margin_top(8);
    card.set_margin_bottom(8);
    card.set_width_request(300);
    card.set_height_request(100);

    let card_content = Box::new(Orientation::Horizontal, 12);
    card_content.set_margin_start(16);
    card_content.set_margin_end(16);
    card_content.set_margin_top(12);
    card_content.set_margin_bottom(12);
    card_content.set_halign(gtk4::Align::Start);
    card_content.set_valign(gtk4::Align::Center);

    let icon = Image::from_icon_name("key-symbolic");
    icon.set_pixel_size(32);
    icon.set_margin_end(8);
    card_content.append(&icon);

    let text_container = Box::new(Orientation::Vertical, 4);
    text_container.set_halign(gtk4::Align::Start);
    text_container.set_valign(gtk4::Align::Center);
    text_container.set_hexpand(true);

    let name_label = Label::new(Some(name));
    name_label.add_css_class("ssh-key-name");
    name_label.add_css_class("title-3");
    name_label.set_halign(gtk4::Align::Start);
    name_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    text_container.append(&name_label);

    let key_info_label = Label::new(Some(key_type));
    key_info_label.add_css_class("ssh-key-info");
    key_info_label.add_css_class("dim-label");
    key_info_label.set_halign(gtk4::Align::Start);
    key_info_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);
    text_container.append(&key_info_label);

    let status_label = Label::new(None);
    status_label.set_halign(gtk4::Align::Start);
    status_label.set_ellipsize(gtk4::pango::EllipsizeMode::End);

    if has_public && has_private {
        status_label.set_text("Paire de clé valide");
        status_label.add_css_class("dim-label");
    } else if !has_private {
        status_label.set_text("Clé privée manquante");
        status_label.add_css_class("error");
    } else if !has_public {
        status_label.set_text("Clé publique manquante");
        status_label.add_css_class("error");
    }

    text_container.append(&status_label);

    card_content.append(&text_container);

    let buttons_container = Box::new(Orientation::Horizontal, 8);
    buttons_container.set_valign(gtk4::Align::Center);

    let info_button = Button::builder()
        .icon_name("dialog-information-symbolic")
        .tooltip_text("Informations")
        .css_classes(vec!["circular".to_string(), "flat".to_string()])
        .width_request(40)
        .height_request(40)
        .build();

    let name_clone_info = name.to_string();
    let type_clone_info = key_type.to_string();
    info_button.connect_clicked(move |button| {
        println!(
            "Affichage des informations de la clé SSH {} ({})",
            name_clone_info, type_clone_info
        );

        match read_key_content(&name_clone_info) {
            Ok((private_content, public_content)) => {
                let dialog = create_info_key_dialog(
                    &name_clone_info,
                    &type_clone_info,
                    public_content.as_deref(),
                    private_content.as_deref(),
                    Rc::clone(&toast_overlay),
                );

                if let Some(parent) = button.root() {
                    dialog.present(Some(&parent));
                }
            }
            Err(e) => {
                eprintln!("Erreur lors de la lecture des clés SSH: {}", e);
            }
        }
    });

    let mut regenerate_button = None;
    if !has_public && has_private {
        let regen_btn = Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Régénérer la clé publique")
            .css_classes(vec![
                "circular".to_string(),
                "flat".to_string(),
                "suggested-action".to_string(),
            ])
            .width_request(40)
            .height_request(40)
            .build();

        let name_clone_regen = name.to_string();
        let type_clone_regen = key_type.to_string();
        let private_path_clone = private_key_path.to_string();
        let refresh_callback_clone = Rc::clone(&refresh_callback);

        regen_btn.connect_clicked(move |_| {
            println!(
                "Régénération de la clé publique pour {} ({})",
                name_clone_regen, type_clone_regen
            );

            match regenerate_public_key(&private_path_clone) {
                Ok(public_key_path) => {
                    println!("Clé publique régénérée avec succès: {}", public_key_path);
                    refresh_callback_clone();
                }
                Err(e) => {
                    eprintln!("Erreur lors de la régénération de la clé publique: {}", e);
                }
            }
        });

        regenerate_button = Some(regen_btn);
    }

    let delete_button = Button::builder()
        .icon_name("user-trash-symbolic")
        .tooltip_text("Supprimer")
        .css_classes(vec![
            "circular".to_string(),
            "flat".to_string(),
            "destructive-action".to_string(),
        ])
        .width_request(40)
        .height_request(40)
        .build();

    let name_clone_delete = name.to_string();
    let type_clone_delete = key_type.to_string();
    let refresh_callback_delete = Rc::clone(&refresh_callback);

    delete_button.connect_clicked(move |button| {
        let dialog = create_delete_key_dialog(&name_clone_delete, &type_clone_delete);

        let name_for_dialog = name_clone_delete.clone();
        let type_for_dialog = type_clone_delete.clone();
        let refresh_callback_for_dialog = Rc::clone(&refresh_callback_delete);

        dialog.connect_response(None, move |_, response| {
            match response {
                "delete" => {
                    println!(
                        "Suppression de la clé SSH {} ({})",
                        name_for_dialog, type_for_dialog
                    );

                    match delete_key_pair(&name_for_dialog) {
                        Ok(_) => {
                            println!("Clé SSH supprimée avec succès");
                            refresh_callback_for_dialog();
                        }
                        Err(e) => {
                            eprintln!("Erreur lors de la suppression de la clé SSH: {}", e);
                        }
                    }
                }
                "cancel" | _ => {
                    println!(
                        "Annulation de la suppression de la clé SSH {} ({})",
                        name_for_dialog, type_for_dialog
                    );
                }
            }
        });

        if let Some(parent) = button.root() {
            dialog.present(Some(&parent));
        }
    });

    buttons_container.append(&info_button);

    if let Some(ref regen_btn) = regenerate_button {
        buttons_container.append(regen_btn);
    }

    buttons_container.append(&delete_button);

    card_content.append(&buttons_container);

    card.set_child(Some(&card_content));

    let name_clone = name.to_string();
    let type_clone = key_type.to_string();
    let gesture = GestureClick::new();
    gesture.connect_pressed(move |_, _, _, _| {
        println!("Sélection de la clé SSH {} ({})", name_clone, type_clone);
    });
    card.add_controller(gesture);

    card
}
