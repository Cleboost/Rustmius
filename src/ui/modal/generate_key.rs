use crate::service::ssh_keys::generate_ssh_key;
use gtk4::{
    Adjustment, Box, Button, Dialog, DropDown, Entry, HeaderBar, Label, Orientation, Scale,
    Spinner, StringList, prelude::*,
};
use gtk4::glib::{self, ControlFlow};
use std::rc::Rc;
use std::thread;
use std::sync::mpsc;

enum GenerationMsg {
    Success(String),
    Error(String),
}

pub fn create_generate_key_dialog(
    refresh_callback: Rc<dyn Fn()>,
    toast_overlay: Rc<libadwaita::ToastOverlay>,
    parent_window: Option<&gtk4::Window>,
) -> Dialog {
    let dialog = Dialog::builder()
        .title("Generate Key")
        .modal(true)
        .resizable(false)
        .decorated(true)
        .build();

    if let Some(parent) = parent_window {
        dialog.set_transient_for(Some(parent));
    }

    let content_box = Box::new(Orientation::Vertical, 12);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Generate SSH Key"))));
    dialog.set_titlebar(Some(&header_bar));

    let name_input = Entry::builder()
        .placeholder_text("Key Name (e.g., github_key)")
        .primary_icon_name("edit-symbolic")
        .build();
    content_box.append(&name_input);

    let key_types = vec!["RSA", "Ed25519", "ECDSA"];
    let key_type_list = StringList::new(&key_types);
    let key_type_dropdown = DropDown::builder().model(&key_type_list).build();

    let key_type_label = Label::new(Some("Key Type:"));
    key_type_label.set_halign(gtk4::Align::Start);
    content_box.append(&key_type_label);
    content_box.append(&key_type_dropdown);

    let length_label = Label::new(Some("Key Length:"));
    length_label.set_halign(gtk4::Align::Start);
    content_box.append(&length_label);

    let adjustment = Adjustment::new(4096.0, 2048.0, 8192.0, 1024.0, 1024.0, 0.0);
    let key_length = Scale::new(Orientation::Horizontal, Some(&adjustment));
    key_length.set_digits(0);
    key_length.set_tooltip_text(Some("Key length in bits (2048/4096/6144/8192)"));
    
    key_length.add_mark(2048.0, gtk4::PositionType::Top, Some("2048"));
    key_length.add_mark(4096.0, gtk4::PositionType::Top, Some("4096"));
    key_length.add_mark(6144.0, gtk4::PositionType::Top, Some("6144"));
    key_length.add_mark(8192.0, gtk4::PositionType::Top, Some("8192"));
    
    content_box.append(&key_length);
    
    let value_label = Label::new(Some("4096 bits"));
    value_label.set_halign(gtk4::Align::Start);
    value_label.add_css_class("dim-label");
    
    let allowed_values = vec![2048, 4096, 6144, 8192];
    
    let value_label_clone = value_label.clone();
    key_length.connect_value_changed(move |scale| {
        let current_value = scale.value() as u32;
        
        let closest_value = allowed_values.iter()
            .min_by_key(|&&x| (x as i32 - current_value as i32).abs())
            .unwrap();
        
        if !allowed_values.contains(&current_value) {
            scale.set_value(*closest_value as f64);
            return;
        }
        
        value_label_clone.set_text(&format!("{} bits", closest_value));
    });
    
    content_box.append(&value_label);

    let email_input = Entry::builder()
        .placeholder_text("Email (optional)")
        .primary_icon_name("mail-symbolic")
        .build();
    content_box.append(&email_input);

    let passphrase_input = Entry::builder()
        .placeholder_text("Passphrase (optional)")
        .primary_icon_name("lock-symbolic")
        .visibility(false)
        .build();
    content_box.append(&passphrase_input);

    let buttons_container = Box::new(Orientation::Horizontal, 12);
    buttons_container.set_halign(gtk4::Align::End);
    buttons_container.set_margin_top(20);

    let cancel_button = Button::builder().label("Cancel").build();
    let generate_button = Button::builder()
        .label("Generate & Save Key")
        .css_classes(vec!["suggested-action"])
        .build();

    let spinner = Spinner::new();
    spinner.set_size_request(20, 20);
    spinner.hide();

    let status_label = Label::new(Some("Generating SSH key..."));
    status_label.add_css_class("dim-label");
    status_label.hide();

    buttons_container.append(&spinner);
    buttons_container.append(&status_label);
    buttons_container.append(&cancel_button);
    buttons_container.append(&generate_button);
    
    content_box.append(&buttons_container);
    dialog.set_child(Some(&content_box));

    let dialog_clone = dialog.clone();
    let name_input_clone = name_input.clone();
    let key_type_dropdown_clone = key_type_dropdown.clone();
    let key_length_clone = key_length.clone();
    let email_input_clone = email_input.clone();
    let passphrase_input_clone = passphrase_input.clone();
    let refresh_callback_clone = Rc::clone(&refresh_callback);
    let toast_overlay_clone = Rc::clone(&toast_overlay);

    generate_button.connect_clicked(move |button| {
        let name = name_input_clone.text().trim().to_string();
        if name.is_empty() {
            let toast = libadwaita::Toast::builder()
                .title("Please enter a key name")
                .build();
            toast_overlay_clone.add_toast(toast);
            return;
        }

        let selected_key_type = key_type_dropdown_clone.selected();
        let key_type = if selected_key_type < key_types.len() as u32 {
            key_types[selected_key_type as usize]
        } else {
            "RSA"
        };

        let key_length_value = key_length_clone.value() as u32;
        let email = email_input_clone.text().trim().to_string();
        let passphrase = passphrase_input_clone.text().trim().to_string();

        button.set_sensitive(false);
        button.set_label("Generating...");
        spinner.show();
        spinner.start();
        status_label.show();

        let (sender, receiver) = mpsc::channel::<GenerationMsg>();

        let name_bg = name.clone();
        let email_bg = email.clone();
        let passphrase_bg = passphrase.clone();
        thread::spawn(move || {
            let result = generate_ssh_key(&name_bg, key_type, key_length_value, &email_bg, &passphrase_bg);
            let _ = match result {
                Ok(_) => sender.send(GenerationMsg::Success(name_bg)),
                Err(e) => sender.send(GenerationMsg::Error(e.to_string())),
            };
        });

        let button_for_idle = button.clone();
        let spinner_for_idle = spinner.clone();
        let status_for_idle = status_label.clone();

        let toast_overlay_main = Rc::clone(&toast_overlay_clone);
        let refresh_callback_main = Rc::clone(&refresh_callback_clone);
        let dialog_main = dialog_clone.clone();
        glib::idle_add_local(move || {
            match receiver.try_recv() {
                Ok(msg) => {
                    spinner_for_idle.hide();
                    spinner_for_idle.stop();
                    status_for_idle.hide();
                    button_for_idle.set_sensitive(true);
                    button_for_idle.set_label("Generate & Save Key");

                    match msg {
                        GenerationMsg::Success(name_val) => {
                            let toast = libadwaita::Toast::builder()
                                .title(&format!("SSH key '{}' generated and saved to ~/.ssh/", name_val))
                                .build();
                            toast_overlay_main.add_toast(toast);
                            refresh_callback_main();
                            dialog_main.close();
                        }
                        GenerationMsg::Error(err) => {
                            let toast = libadwaita::Toast::builder()
                                .title(&format!("Failed to generate SSH key: {}", err))
                                .build();
                            toast_overlay_main.add_toast(toast);
                        }
                    }

                    ControlFlow::Break
                }
                Err(mpsc::TryRecvError::Empty) => ControlFlow::Continue,
                Err(mpsc::TryRecvError::Disconnected) => ControlFlow::Break,
            }
        });
    });

    let dialog_clone = dialog.clone();
    cancel_button.connect_clicked(move |_| {
        dialog_clone.close();
    });

    dialog
}
