use gtk4::prelude::*;
use crate::config_observer::SshHost;

pub fn show_server_dialog<F>(parent: &gtk4::Window, initial_host: Option<&SshHost>, on_save: F)
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
    let user_entry = gtk4::Entry::builder().placeholder_text("User (default: root)").build();
    let pass_entry = gtk4::PasswordEntry::builder()
        .placeholder_text("Password (leave empty to keep current or no password)")
        .show_peek_icon(true)
        .build();

    if let Some(host) = initial_host {
        alias_entry.set_text(&host.alias);
        host_entry.set_text(&host.hostname);
        if let Some(ref user) = host.user {
            user_entry.set_text(user);
        }
    }

    content.append(&gtk4::Label::builder().label("Alias").halign(gtk4::Align::Start).build());
    content.append(&alias_entry);
    content.append(&gtk4::Label::builder().label("Hostname").halign(gtk4::Align::Start).build());
    content.append(&host_entry);
    content.append(&gtk4::Label::builder().label("User").halign(gtk4::Align::Start).build());
    content.append(&user_entry);
    content.append(&gtk4::Label::builder().label("Password").halign(gtk4::Align::Start).build());
    content.append(&pass_entry);

    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
    dialog.add_button(if initial_host.is_some() { "Save" } else { "Add" }, gtk4::ResponseType::Ok);

    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Ok {
            let host = SshHost {
                alias: alias_entry.text().to_string(),
                hostname: host_entry.text().to_string(),
                user: Some(user_entry.text().to_string()).filter(|s| !s.is_empty()),
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
