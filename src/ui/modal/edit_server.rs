use libadwaita::{prelude::*, Dialog};
use libadwaita::{ComboRow, EntryRow};
use gtk4::{Box as GtkBox, Orientation, StringList, Button, DropDown};
use crate::service::{SshServer, load_ssh_keys};

pub fn create_edit_server_dialog(server: &SshServer, on_save: Option<std::boxed::Box<dyn Fn() + 'static>>) -> Dialog {
    println!("Creating edit dialog for server: {}", server.name);
    let dialog = Dialog::builder()
        .title(&format!("Edit Server: {}", server.name))
        .build();

    // Créer les boutons d'action
    let cancel_button = Button::builder()
        .label("Annuler")
        .build();

    let save_button = Button::builder()
        .label("Sauvegarder")
        .css_classes(vec!["suggested-action"])
        .build();

    // Créer le contenu principal
    let content_box = GtkBox::new(Orientation::Vertical, 12);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    // Champ pour le nom du serveur
    let name_row = EntryRow::builder()
        .title("Server Name")
        .text(&server.name)
        .build();
    content_box.append(&name_row);

    // Champ pour l'hostname/IP
    let hostname_row = EntryRow::builder()
        .title("Hostname/IP")
        .text(server.hostname.as_deref().unwrap_or(""))
        .build();
    content_box.append(&hostname_row);

    // Champ pour l'utilisateur
    let user_row = EntryRow::builder()
        .title("User")
        .text(server.user.as_deref().unwrap_or(""))
        .build();
    content_box.append(&user_row);

    // Champ pour le port
    let port_row = EntryRow::builder()
        .title("Port")
        .text(server.port.map(|p| p.to_string()).unwrap_or_else(|| "22".to_string()))
        .build();
    content_box.append(&port_row);

    // Créer un DropDown pour la sélection de la clé SSH
    let ssh_key_dropdown = DropDown::new(None::<StringList>, None::<&gtk4::Expression>);
    
    // Créer une ComboRow pour l'affichage
    let ssh_key_row = ComboRow::builder()
        .title("SSH Key")
        .use_subtitle(false)
        .build();

    // Charger les clés SSH disponibles
    let ssh_keys = match load_ssh_keys() {
        Ok(keys) => {
            println!("Loaded SSH keys:");
            for key in &keys {
                println!("  - {} (type: {}, path: {})", key.name, key.key_type, key.file_path);
            }
            keys
        },
        Err(e) => {
            println!("Error loading SSH keys: {}", e);
            vec![]
        },
    };

    // Créer la liste des clés pour le ComboRow
    let key_list = StringList::new(&[]);
    
    // Stocker les noms des clés pour la recherche
    let mut key_names = vec!["None".to_string()];
    
    // Ajouter "None" comme première option
    key_list.append("None");
    
    // Ajouter les clés disponibles
    for key in &ssh_keys {
        key_list.append(&key.name);
        key_names.push(key.name.clone());
    }
    
    // Configurer le DropDown
    ssh_key_dropdown.set_model(Some(&key_list));
    
    // Connecter le signal de changement de sélection pour debug
    ssh_key_dropdown.connect_selected_notify({
        let ssh_key_dropdown = ssh_key_dropdown.clone();
        move |_| {
            let selected = ssh_key_dropdown.selected();
            println!("SSH Key selection changed to index: {}", selected);
        }
    });
    
    // Sélectionner la clé SSH actuelle si elle existe
    if let Some(current_key) = &server.identity_file {
        // Extraire le nom de la clé du chemin (ex: /home/user/.ssh/id_rsa -> id_rsa)
        let key_name = current_key.split('/').last().unwrap_or("None");
        println!("Looking for key: {}", key_name);
        
        // Chercher la clé dans la liste des noms
        if let Some(index) = key_names.iter().position(|name| name == key_name) {
            ssh_key_dropdown.set_selected(index as u32);
            println!("Selected key '{}' at index: {}", key_name, index);
        } else {
            println!("Key '{}' not found in available keys, selecting None", key_name);
            ssh_key_dropdown.set_selected(0);
        }
    } else {
        // Sélectionner "None" par défaut si aucune clé n'est configurée
        ssh_key_dropdown.set_selected(0);
        println!("No SSH key configured, selecting None");
    }

    // Ajouter le DropDown à la ComboRow
    ssh_key_row.add_suffix(&ssh_key_dropdown);
    content_box.append(&ssh_key_row);

    // Ajouter les boutons en bas
    let button_box = GtkBox::new(Orientation::Horizontal, 12);
    button_box.set_halign(gtk4::Align::End);
    button_box.set_margin_top(20);
    button_box.append(&cancel_button);
    button_box.append(&save_button);
    content_box.append(&button_box);

    // Connecter les boutons
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
            // Récupérer les valeurs des champs
            let new_name = name_row.text();
            let new_hostname = hostname_row.text();
            let new_user = user_row.text();
            let new_port = port_row.text();
            let selected_key_index = ssh_key_dropdown.selected();
            
            // Récupérer le nom de la clé sélectionnée
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

            // Sauvegarder les modifications dans le fichier SSH config
            if let Err(e) = save_server_config(&server_name, &new_name, &new_hostname, &new_user, &new_port, &selected_key) {
                eprintln!("Erreur lors de la sauvegarde: {}", e);
            } else {
                println!("Configuration sauvegardée avec succès");
                // Recharger l'UI pour afficher les changements
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
    
    // Lire le contenu actuel du fichier
    let content = fs::read_to_string(&ssh_config_path)?;
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    
    // Trouver et modifier la section du serveur
    let mut in_target_section = false;
    let mut section_start = 0;
    let mut section_end = 0;
    
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("Host ") && line.trim().ends_with(&old_name) {
            in_target_section = true;
            section_start = i;
        } else if in_target_section && line.trim().starts_with("Host ") {
            section_end = i;
            break;
        }
    }
    
    if in_target_section {
        if section_end == 0 {
            section_end = lines.len();
        }
        
        // Créer la nouvelle section
        let mut new_section = vec![format!("Host {}", new_name)];
        
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
        
        // Remplacer l'ancienne section par la nouvelle
        lines.splice(section_start..section_end, new_section);
        
        // Écrire le fichier modifié
        let new_content = lines.join("\n");
        fs::write(&ssh_config_path, new_content)?;
        
        println!("Fichier SSH config mis à jour avec succès");
    } else {
        return Err(format!("Section 'Host {}' introuvable dans le fichier config", old_name).into());
    }
    
    Ok(())
}