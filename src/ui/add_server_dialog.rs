#![allow(deprecated)]
use gtk4::prelude::*;
use crate::config_observer::SshHost;
use crate::ui::ssh_keys::load_ssh_keys;

pub fn show_server_dialog<F>(
    parent: &gtk4::Window, 
    initial_host: Option<&SshHost>, 
    existing_aliases: Vec<String>,
    on_save: F
)
where F: Fn(SshHost, String) + 'static
{
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(if initial_host.is_some() { "Edit Server" } else { "Add New Server" })
        .default_width(400)
        .build();

    let content = dialog.content_area();
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_spacing(12);

    let alias_entry = gtk4::Entry::builder().placeholder_text("Alias (e.g. My VPS)").build();
    let host_entry = gtk4::Entry::builder().placeholder_text("Hostname or IP").build();
    let port_entry = gtk4::Entry::builder().placeholder_text("Port (default: 22)").build();
    let user_entry = gtk4::Entry::builder().placeholder_text("User (default: root)").build();
    let pass_entry = gtk4::PasswordEntry::builder()
        .placeholder_text("Password (leave empty to keep current or no password)")
        .show_peek_icon(true)
        .build();

    let error_label = gtk4::Label::builder()
        .label("Alias already exists!")
        .halign(gtk4::Align::Start)
        .visible(false)
        .build();
    error_label.add_css_class("error");

    if let Some(host) = initial_host {
        alias_entry.set_text(&host.alias);
        host_entry.set_text(&host.hostname);
        if let Some(ref user) = host.user {
            user_entry.set_text(user);
        }
        if let Some(port) = host.port {
            port_entry.set_text(&port.to_string());
        }
    }

    let keys = load_ssh_keys();
    let key_model = gtk4::StringList::new(&[]);
    key_model.append("None (Default Auth)");
    for k in &keys {
        key_model.append(&k.name);
    }
    let key_dropdown = gtk4::DropDown::new(Some(key_model), gtk4::Expression::NONE);

    if let Some(host) = initial_host {
        if let Some(ref id_file) = host.identity_file {
            // Normalize the stored path (which may contain `~`) to an absolute
            // path so it compares correctly with the absolute priv_path from
            // load_ssh_keys().
            let id_file_expanded = crate::config_observer::expand_tilde(id_file);
            for (i, k) in keys.iter().enumerate() {
                if k.priv_path == id_file_expanded {
                    key_dropdown.set_selected((i + 1) as u32);
                    break;
                }
            }
        }
    }

    content.append(&gtk4::Label::builder().label("Alias").halign(gtk4::Align::Start).build());
    content.append(&alias_entry);
    content.append(&error_label);
    content.append(&gtk4::Label::builder().label("Hostname").halign(gtk4::Align::Start).build());
    content.append(&host_entry);
    content.append(&gtk4::Label::builder().label("Port").halign(gtk4::Align::Start).build());
    content.append(&port_entry);
    content.append(&gtk4::Label::builder().label("User").halign(gtk4::Align::Start).build());
    content.append(&user_entry);
    content.append(&gtk4::Label::builder().label("Password").halign(gtk4::Align::Start).build());
    content.append(&pass_entry);
    content.append(&gtk4::Label::builder().label("SSH Key").halign(gtk4::Align::Start).build());
    content.append(&key_dropdown);

    let ok_button = dialog.add_button(if initial_host.is_some() { "Save" } else { "Add" }, gtk4::ResponseType::Ok);
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);

    let existing_aliases = Rc::new(existing_aliases);
    let initial_alias = initial_host.map(|h| h.alias.to_lowercase());
    
    let alias_entry_clone = alias_entry.clone();
    let error_label_clone = error_label.clone();
    let ok_button_clone = ok_button.clone();
    let existing_aliases_clone = existing_aliases.clone();
    
    alias_entry.connect_changed(move |e| {
        let text = e.text().to_string().trim().to_lowercase();
        let is_duplicate = existing_aliases_clone.contains(&text) && Some(text.clone()) != initial_alias;
        
        error_label_clone.set_visible(is_duplicate);
        ok_button_clone.set_sensitive(!is_duplicate && !text.is_empty());
    });

    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Ok {
            let selected_key_idx = key_dropdown.selected();
            let identity_file = if selected_key_idx > 0 {
                let key = &keys[(selected_key_idx - 1) as usize];
                Some(key.priv_path.to_string_lossy().to_string())
            } else {
                None
            };
            
            let host = SshHost {
                alias: alias_entry_clone.text().to_string().trim().to_string(),
                hostname: host_entry.text().to_string().trim().to_string(),
                user: Some(user_entry.text().to_string().trim().to_string()).filter(|s| !s.is_empty()),
                port: port_entry.text().to_string().trim().parse::<u16>().ok(),
                identity_file,
            };
            let password = pass_entry.text().to_string();
            
            if !host.alias.is_empty() && !host.hostname.is_empty() {
                on_save(host, password);
            }
        }
        d.close();
    });

    dialog.present();
}

use std::rc::Rc;
