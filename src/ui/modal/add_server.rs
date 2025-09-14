use crate::service::ssh_keys::load_ssh_keys;
use dirs;
use gtk4::{
    Box, Button, Dialog, DropDown, Entry, HeaderBar, Label, Orientation, StringList, StringObject,
    prelude::*,
};
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;

pub fn create_add_server_dialog(on_save: Option<std::boxed::Box<dyn Fn() + 'static>>) -> Dialog {
    let dialog = Dialog::builder().title("Add Server").build();

    let content_box = Box::new(Orientation::Vertical, 12);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let header_bar = HeaderBar::new();
    header_bar.set_title_widget(Some(&Label::new(Some("Add Server"))));
    dialog.set_titlebar(Some(&header_bar));

    let name_input = Entry::builder()
        .placeholder_text("Name")
        .primary_icon_name("edit-symbolic")
        .build();
    content_box.append(&name_input);

    let ip_input = Entry::builder()
        .placeholder_text("IP")
        .primary_icon_name("network-server-symbolic")
        .build();
    content_box.append(&ip_input);

    let user_input = Entry::builder()
        .placeholder_text("User")
        .primary_icon_name("avatar-default-symbolic")
        .build();
    content_box.append(&user_input);

    let port_input = Entry::builder()
        .placeholder_text("Port")
        .text("22")
        .primary_icon_name("network-wired-symbolic")
        .build();
    content_box.append(&port_input);

    let ssh_key_label = Label::new(Some("SSH Key:"));
    ssh_key_label.set_halign(gtk4::Align::Start);
    content_box.append(&ssh_key_label);

    let ssh_key_dropdown = DropDown::new(None::<StringList>, None::<&gtk4::Expression>);

    let ssh_keys = match load_ssh_keys() {
        Ok(keys) => keys,
        Err(_e) => {
            vec![]
        }
    };

    let key_list = StringList::new(&[]);
    let mut key_names = vec!["None".to_string()];

    key_list.append("None");
    for key in &ssh_keys {
        key_list.append(&key.name);
        key_names.push(key.name.clone());
    }

    ssh_key_dropdown.set_model(Some(&key_list));
    ssh_key_dropdown.set_selected(0);

    ssh_key_dropdown.connect_selected_notify({
        let _ssh_key_dropdown = ssh_key_dropdown.clone();
        move |_| {}
    });

    content_box.append(&ssh_key_dropdown);

    let cancel_button = Button::builder()
        .label("Cancel")
        .css_classes(vec!["destructive-action"])
        .build();

    let export_key_button = Button::builder()
        .label("Export Key")
        .css_classes(vec!["flat"])
        .build();

    let save_button = Button::builder()
        .label("Save")
        .css_classes(vec!["suggested-action"])
        .build();
    save_button.set_sensitive(false);

    let button_box = Box::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.append(&export_key_button);
    button_box.append(&cancel_button);
    button_box.append(&save_button);
    content_box.append(&button_box);

    dialog.set_child(Some(&content_box));

    {
        let dialog = dialog.clone();
        cancel_button.connect_clicked(move |_| {
            dialog.close();
        });
    }

    let dialog_clone_export = dialog.clone();
    let ip_input_clone = ip_input.clone();
    let user_input_clone = user_input.clone();
    let ssh_drop_clone = ssh_key_dropdown.clone();

    export_key_button.connect_clicked(move |_| {
        let host = ip_input_clone.text();
        let user = user_input_clone.text();
        let selected = ssh_drop_clone.selected();
        if selected == 0 {
            return;
        }
        let key_name = ssh_drop_clone
            .model()
            .and_then(|m| m.item(selected))
            .and_then(|obj| obj.downcast::<StringObject>().ok())
            .map(|o| o.string())
            .unwrap_or_default();

        let pub_key_path = format!(
            "{}/.ssh/{}.pub",
            dirs::home_dir().unwrap().display(),
            key_name
        );
        let export_dialog = crate::ui::modal::export_key::create_export_key_dialog(
            &dialog_clone_export,
            &user,
            &host,
            &pub_key_path,
        );
        export_dialog.present();
    });

    let update_state = Rc::new({
        let name_input = name_input.clone();
        let ip_input = ip_input.clone();
        let user_input = user_input.clone();
        let save_button = save_button.clone();
        let ssh_key_dropdown = ssh_key_dropdown.clone();
        move || {
            let name_ok = !name_input.text().trim().is_empty();
            let ip_ok = !ip_input.text().trim().is_empty();
            let user_ok = !user_input.text().trim().is_empty();
            let key_ok = ssh_key_dropdown.selected() > 0;
            let enable = name_ok && ip_ok && user_ok && key_ok;
            save_button.set_sensitive(enable);
        }
    });

    {
        let update_state = Rc::clone(&update_state);
        name_input.connect_changed(move |_| {
            update_state();
        });
    }
    {
        let update_state = Rc::clone(&update_state);
        ip_input.connect_changed(move |_| {
            update_state();
        });
    }
    {
        let update_state = Rc::clone(&update_state);
        user_input.connect_changed(move |_| {
            update_state();
        });
    }
    {
        let update_state = Rc::clone(&update_state);
        let ssh_key_dropdown = ssh_key_dropdown.clone();
        ssh_key_dropdown.connect_selected_notify(move |_| {
            update_state();
        });
    }
    update_state();

    save_button.connect_clicked({
        let dialog = dialog.clone();
        let name_input = name_input.clone();
        let ip_input = ip_input.clone();
        let user_input = user_input.clone();
        let port_input = port_input.clone();
        let ssh_key_dropdown = ssh_key_dropdown.clone();
        let ssh_keys = ssh_keys.clone();
        move |_| {
            // Allow users to enter a friendly/display name after the real host token.
            // Example: "Domoticz [PAPA]" -> host_token = "Domoticz", display_name = "[PAPA]"
            let name_raw = name_input.text().trim().to_string();
            let mut name_parts = name_raw.split_whitespace();
            let host_token = name_parts.next().unwrap_or("").to_string();
            let display_name = {
                let rest: Vec<&str> = name_parts.collect();
                if rest.is_empty() {
                    None
                } else {
                    Some(rest.join(" "))
                }
            };

            let hostname = ip_input.text().trim().to_string();
            let user = user_input.text().trim().to_string();
            let port = port_input.text().trim().to_string();

            // Validate using host_token (the actual SSH Host token) rather than the full display string
            if host_token.is_empty() || hostname.is_empty() || user.is_empty() {
                return;
            }

            let selected_index = ssh_key_dropdown.selected();
            let selected_key_name = if selected_index > 0 {
                let key_index = (selected_index - 1) as usize;
                if key_index < ssh_keys.len() {
                    Some(ssh_keys[key_index].name.clone())
                } else {
                    None
                }
            } else {
                None
            };

            let ssh_dir = dirs::home_dir().unwrap_or_default().join(".ssh");
            let config_path: PathBuf = ssh_dir.join("config");

            if !ssh_dir.exists() {
                if let Err(e) = fs::create_dir_all(&ssh_dir) {
                    eprintln!("Failed to create ~/.ssh directory: {}", e);
                    return;
                }
            }

            if !config_path.exists() {
                if let Err(e) = fs::write(&config_path, "") {
                    eprintln!("Failed to create ~/.ssh/config: {}", e);
                    return;
                }
            }

            let existing = match fs::read_to_string(&config_path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Failed to read ~/.ssh/config: {}", e);
                    return;
                }
            };

            let mut lines: Vec<String> = existing.lines().map(|s| s.to_string()).collect();

            let mut new_section: Vec<String> = Vec::new();
            // Use the actual SSH token as Host (no spaces)
            new_section.push(format!("Host {}", host_token));
            // Persist the user-friendly display name as a directive for the app to use
            if let Some(ref disp) = display_name {
                if !disp.is_empty() {
                    new_section.push(format!("  DisplayName {}", disp));
                }
            }
            new_section.push(format!("  HostName {}", hostname));
            new_section.push(format!("  User {}", user));
            if !port.is_empty() && port != "22" {
                new_section.push(format!("  Port {}", port));
            }
            if let Some(key_name) = selected_key_name.as_ref() {
                new_section.push(format!("  IdentityFile ~/.ssh/{}", key_name));
            }

            let mut section_start: Option<usize> = None;
            let mut section_end: usize = lines.len();
            for (i, line) in lines.iter().enumerate() {
                let trimmed = line.trim();
                if trimmed.starts_with("Host ") {
                    let host_val = trimmed.strip_prefix("Host ").unwrap_or("");
                    // Compare only the first token of the existing Host line (the real ssh host token)
                    let existing_token = host_val.split_whitespace().next().unwrap_or("");
                    if section_start.is_none() && existing_token == host_token {
                        section_start = Some(i);
                    } else if section_start.is_some() {
                        section_end = i;
                        break;
                    }
                }
            }

            if let Some(start) = section_start {
                lines.splice(start..section_end, new_section);
            } else {
                if !lines.is_empty() && !existing.ends_with('\n') {
                    lines.push(String::new());
                }
                lines.extend(new_section);
                lines.push(String::new());
            }

            let new_content = lines.join("\n");
            if let Err(e) = fs::write(&config_path, new_content) {
                eprintln!("Failed to write ~/.ssh/config: {}", e);
                return;
            }

            if let Some(cb) = &on_save {
                cb();
            }
            dialog.close();
        }
    });

    dialog
}
