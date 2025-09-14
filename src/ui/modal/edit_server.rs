use crate::service::{SshServer, load_ssh_keys};
use gtk4::{Box as GtkBox, Button, DropDown, Orientation, StringList};
use libadwaita::{ComboRow, EntryRow};
use libadwaita::{Dialog, prelude::*};

pub fn create_edit_server_dialog(
    server: &SshServer,
    on_save: Option<std::boxed::Box<dyn Fn() + 'static>>,
) -> Dialog {
    println!("Creating edit dialog for server: {}", server.name);
    let dialog = Dialog::builder()
        .title(&format!("Edit Server: {}", server.name))
        .build();

    let cancel_button = Button::builder().label("Annuler").build();

    let save_button = Button::builder()
        .label("Sauvegarder")
        .css_classes(vec!["suggested-action"])
        .build();

    let content_box = GtkBox::new(Orientation::Vertical, 12);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let name_row = EntryRow::builder()
        .title("Server Name")
        .text(&server.name)
        .build();
    content_box.append(&name_row);

    let hostname_row = EntryRow::builder()
        .title("Hostname/IP")
        .text(server.hostname.as_deref().unwrap_or(""))
        .build();
    content_box.append(&hostname_row);

    let user_row = EntryRow::builder()
        .title("User")
        .text(server.user.as_deref().unwrap_or(""))
        .build();
    content_box.append(&user_row);

    let port_row = EntryRow::builder()
        .title("Port")
        .text(
            server
                .port
                .map(|p| p.to_string())
                .unwrap_or_else(|| "22".to_string()),
        )
        .build();
    content_box.append(&port_row);

    let ssh_key_dropdown = DropDown::new(None::<StringList>, None::<&gtk4::Expression>);

    let ssh_key_row = ComboRow::builder()
        .title("SSH Key")
        .use_subtitle(false)
        .build();

    let ssh_keys = match load_ssh_keys() {
        Ok(keys) => {
            println!("Loaded SSH keys:");
            for key in &keys {
                println!(
                    "  - {} (type: {}, path: {})",
                    key.name, key.key_type, key.file_path
                );
            }
            keys
        }
        Err(e) => {
            println!("Error loading SSH keys: {}", e);
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

    ssh_key_dropdown.connect_selected_notify({
        let ssh_key_dropdown = ssh_key_dropdown.clone();
        move |_| {
            let selected = ssh_key_dropdown.selected();
            println!("SSH Key selection changed to index: {}", selected);
        }
    });

    if let Some(current_key) = &server.identity_file {
        let key_name = current_key.split('/').last().unwrap_or("None");
        println!("Looking for key: {}", key_name);

        if let Some(index) = key_names.iter().position(|name| name == key_name) {
            ssh_key_dropdown.set_selected(index as u32);
            println!("Selected key '{}' at index: {}", key_name, index);
        } else {
            println!(
                "Key '{}' not found in available keys, selecting None",
                key_name
            );
            ssh_key_dropdown.set_selected(0);
        }
    } else {
        ssh_key_dropdown.set_selected(0);
        println!("No SSH key configured, selecting None");
    }

    ssh_key_row.add_suffix(&ssh_key_dropdown);
    content_box.append(&ssh_key_row);

    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(20);
    button_box.append(&cancel_button);
    button_box.append(&save_button);
    content_box.append(&button_box);

    cancel_button.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            println!("Cancel button clicked - closing dialog");
            dialog.close();
        }
    });

    save_button.connect_clicked({
        let name_row = name_row.clone();
        let hostname_row = hostname_row.clone();
        let user_row = user_row.clone();
        let port_row = port_row.clone();
        let ssh_key_dropdown = ssh_key_dropdown.clone();
        let ssh_keys = ssh_keys.clone();
        let dialog = dialog.clone();
        let server_name = server.name.clone();
        let on_save_clone = on_save;
        move |_| {
            let new_name = name_row.text();
            let new_hostname = hostname_row.text();
            let new_user = user_row.text();
            let new_port = port_row.text();
            let selected_key_index = ssh_key_dropdown.selected();

            let selected_key = if selected_key_index > 0 {
                let key_index = (selected_key_index - 1) as usize;
                if key_index < ssh_keys.len() {
                    ssh_keys[key_index].name.clone()
                } else {
                    "None".to_string()
                }
            } else {
                "None".to_string()
            };

            println!("Saving server changes:");
            println!("  Name: {}", new_name);
            println!("  Hostname: {}", new_hostname);
            println!("  User: {}", new_user);
            println!("  Port: {}", new_port);
            println!("  SSH Key: {}", selected_key);

            if let Err(e) = save_server_config(
                &server_name,
                &new_name,
                &new_hostname,
                &new_user,
                &new_port,
                &selected_key,
            ) {
                eprintln!("Erreur lors de la sauvegarde: {}", e);
            } else {
                println!("Configuration sauvegardée avec succès");
                if let Some(refresh_fn) = &on_save_clone {
                    println!("Calling refresh function...");
                    refresh_fn();
                    println!("Refresh function called successfully");
                } else {
                    println!("No refresh function provided");
                }
            }

            dialog.close();
        }
    });

    dialog.set_child(Some(&content_box));

    dialog
}

fn save_server_config(
    old_name: &str,
    new_name: &str,
    new_hostname: &str,
    new_user: &str,
    new_port: &str,
    new_ssh_key: &str,
) -> Result<(), std::boxed::Box<dyn std::error::Error>> {
    use std::fs;

    let ssh_config_path = dirs::home_dir()
        .ok_or("Impossible de trouver le répertoire home")?
        .join(".ssh/config");

    if !ssh_config_path.exists() {
        return Err("Fichier ~/.ssh/config introuvable".into());
    }

    let content = fs::read_to_string(&ssh_config_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();

    // Find the section by comparing the first token of the Host line (the real SSH token).
    let mut section_start: Option<usize> = None;
    let mut section_end: usize = lines.len();
    let mut existing_display: Option<String> = None;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("Host ") {
            let host_val = trimmed.strip_prefix("Host ").unwrap_or("");
            let existing_token = host_val.split_whitespace().next().unwrap_or("");
            if existing_token == old_name {
                section_start = Some(i);

                // scan section for an existing DisplayName and determine its end
                let mut j = i + 1;
                while j < lines.len() {
                    let t = lines[j].trim();
                    if t.starts_with("Host ") {
                        section_end = j;
                        break;
                    }
                    let t_lower = t.to_lowercase();
                    if t_lower.starts_with("displayname ") {
                        // preserve existing DisplayName value
                        existing_display = Some(t["DisplayName ".len()..].trim().to_string());
                    }
                    j += 1;
                }
                // if we didn't find another Host, section_end remains lines.len()
                break;
            }
        }
    }

    if let Some(start) = section_start {
        // Parse new_name to extract token and optional display name entered by user
        let name_raw = new_name.trim();
        let mut parts = name_raw.split_whitespace();
        let new_token = parts.next().unwrap_or("").to_string();
        let rest: Vec<&str> = parts.collect();
        let new_display = if rest.is_empty() {
            None
        } else {
            Some(rest.join(" "))
        };

        // If user didn't provide a new display name, keep the existing one (if any)
        let final_display = new_display.or(existing_display);

        let mut new_section: Vec<String> = Vec::new();
        new_section.push(format!("Host {}", new_token));

        if let Some(ref d) = final_display {
            if !d.trim().is_empty() {
                new_section.push(format!("  DisplayName {}", d));
            }
        }

        if !new_hostname.is_empty() {
            new_section.push(format!("  HostName {}", new_hostname));
        }

        if !new_user.is_empty() {
            new_section.push(format!("  User {}", new_user));
        }

        if !new_port.is_empty() && new_port != "22" {
            new_section.push(format!("  Port {}", new_port));
        }

        if new_ssh_key != "None" && !new_ssh_key.is_empty() {
            new_section.push(format!("  IdentityFile ~/.ssh/{}", new_ssh_key));
        }

        // Replace the old section with the new one
        lines.splice(start..section_end, new_section);

        let new_content = lines.join("\n");
        fs::write(&ssh_config_path, new_content)?;

        println!("Fichier SSH config mis à jour avec succès");
    } else {
        return Err(format!(
            "Section 'Host {}' introuvable dans le fichier config",
            old_name
        )
        .into());
    }

    Ok(())
}
