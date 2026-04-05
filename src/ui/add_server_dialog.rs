use gtk4::prelude::*;
use crate::config_observer::SshHost;

pub fn show_add_server_dialog<F>(parent: &gtk4::Window, on_add: F)
where F: Fn(SshHost) + 'static
{
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Add New Server")
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

    content.append(&gtk4::Label::new(Some("Alias")));
    content.append(&alias_entry);
    content.append(&gtk4::Label::new(Some("Hostname")));
    content.append(&host_entry);
    content.append(&gtk4::Label::new(Some("User")));
    content.append(&user_entry);

    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
    dialog.add_button("Add", gtk4::ResponseType::Ok);

    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Ok {
            let host = SshHost {
                alias: alias_entry.text().to_string(),
                hostname: host_entry.text().to_string(),
                user: Some(user_entry.text().to_string()).filter(|s| !s.is_empty()),
            };
            if !host.alias.is_empty() && !host.hostname.is_empty() {
                on_add(host);
            }
        }
        d.close();
    });

    dialog.present();
}
