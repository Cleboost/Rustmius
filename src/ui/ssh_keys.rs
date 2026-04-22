#![allow(deprecated)]
use gtk4::prelude::*;
use gtk4::glib;
use std::rc::Rc;
use std::cell::RefCell;
use directories::UserDirs;
use std::path::PathBuf;
use crate::config_observer::load_hosts;

fn is_valid_key_name(name: &str) -> bool {
    if name.is_empty() || name.contains('\0') {
        return false;
    }
    let p = std::path::Path::new(name);
    p.components().count() == 1
        && p.file_name().map(|n| n == std::ffi::OsStr::new(name)).unwrap_or(false)
}

fn make_error_alert(parent: Option<&gtk4::Window>, title: &str, secondary: &str) -> gtk4::MessageDialog {
    let builder = gtk4::MessageDialog::builder()
        .modal(true)
        .message_type(gtk4::MessageType::Error)
        .buttons(gtk4::ButtonsType::Ok)
        .text(title)
        .secondary_text(secondary);
    let alert = if let Some(w) = parent {
        builder.transient_for(w).build()
    } else {
        builder.build()
    };
    alert.connect_response(|a, _| a.close());
    alert
}

#[derive(Clone)]
pub struct SshKeyPair {
    pub name: String,
    pub pub_path: PathBuf,
    pub priv_path: PathBuf,
}

fn get_ssh_dir() -> Option<PathBuf> {
    UserDirs::new().map(|dirs| dirs.home_dir().join(".ssh"))
}

pub fn load_ssh_keys() -> Vec<SshKeyPair> {
    let mut keys = Vec::new();
    if let Some(ssh_dir) = get_ssh_dir()
        && let Ok(entries) = std::fs::read_dir(&ssh_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("pub") {
                    let mut priv_path = path.clone();
                    priv_path.set_extension("");
                    if priv_path.exists() {
                        let name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                        keys.push(SshKeyPair {
                            name,
                            pub_path: path,
                            priv_path,
                        });
                    }
                }
            }
        }
    keys.sort_by(|a, b| a.name.cmp(&b.name));
    keys
}

pub fn build_ssh_keys_ui(window: &gtk4::ApplicationWindow) -> gtk4::Box {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
    main_box.set_margin_top(24); main_box.set_margin_bottom(24);
    main_box.set_margin_start(24); main_box.set_margin_end(24);

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    let title = gtk4::Label::builder().label("SSH Keys").halign(gtk4::Align::Start).hexpand(true).build();
    title.add_css_class("title-1");
    let gen_btn = gtk4::Button::from_icon_name("list-add-symbolic");
    gen_btn.set_tooltip_text(Some("Generate New Key"));
    gen_btn.add_css_class("suggested-action");
    let import_btn = gtk4::Button::from_icon_name("document-import-symbolic");
    import_btn.set_tooltip_text(Some("Import Key"));
    import_btn.add_css_class("flat");
    let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some("Refresh"));

    header_box.append(&title);
    header_box.append(&refresh_btn);
    header_box.append(&import_btn);
    header_box.append(&gen_btn);

    main_box.append(&header_box);

    let list_box = gtk4::ListBox::new();
    list_box.set_selection_mode(gtk4::SelectionMode::None);
    list_box.add_css_class("boxed-list");
    let scrolled = gtk4::ScrolledWindow::builder()
        .child(&list_box)
        .vexpand(true)
        .build();
    main_box.append(&scrolled);

    let list_box_rc = Rc::new(list_box);
    let window_rc = window.clone();

    let refresh_ui: Rc<RefCell<Option<Rc<dyn Fn()>>>> = Rc::new(RefCell::new(None));

    let do_refresh = {
        let lb = list_box_rc.clone();
        let win = window_rc.clone();
        let rwh = Rc::downgrade(&refresh_ui);
        Rc::new(move || {
            while let Some(child) = lb.first_child() {
                lb.remove(&child);
            }

            let keys = load_ssh_keys();
            if keys.is_empty() {
                let empty_lbl = gtk4::Label::new(Some("No SSH keys found in ~/.ssh/"));
                empty_lbl.set_margin_top(24);
                empty_lbl.set_margin_bottom(24);
                empty_lbl.add_css_class("dim-label");
                lb.append(&empty_lbl);
            } else {
                for key in keys {
                    let row = gtk4::ListBoxRow::new();
                    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
                    hbox.set_margin_start(12); hbox.set_margin_end(12);
                    hbox.set_margin_top(12); hbox.set_margin_bottom(12);
                    let icon = gtk4::Image::from_icon_name("network-vpn-symbolic");
                    icon.set_pixel_size(24);
                    let name_lbl = gtk4::Label::new(Some(&key.name));
                    name_lbl.set_halign(gtk4::Align::Start);
                    name_lbl.set_hexpand(true);

                    let deploy_btn = gtk4::Button::from_icon_name("document-send-symbolic");
                    deploy_btn.set_tooltip_text(Some("Deploy to Server"));
                    deploy_btn.add_css_class("flat");

                    let del_btn = gtk4::Button::from_icon_name("user-trash-symbolic");
                    del_btn.set_tooltip_text(Some("Delete Key"));
                    del_btn.add_css_class("destructive-action");
                    del_btn.add_css_class("flat");

                    let key_clone1 = key.clone();
                    let key_clone2 = key.clone();
                    let w_clone = win.clone();
                    let w_clone2 = win.clone();
                    let handle = rwh.clone();

                    del_btn.connect_clicked(move |_| {
                        let dialog = gtk4::MessageDialog::builder()
                            .transient_for(&w_clone)
                            .modal(true)
                            .message_type(gtk4::MessageType::Warning)
                            .buttons(gtk4::ButtonsType::OkCancel)
                            .text(format!("Delete key '{}'?", key_clone1.name))
                            .secondary_text("This action cannot be undone and will delete both public and private key files.")
                            .build();

                        let p1 = key_clone1.pub_path.clone();
                        let p2 = key_clone1.priv_path.clone();
                        let h = handle.clone();
                        let w_del = w_clone.clone();

                        dialog.connect_response(move |d, res| {
                            if res == gtk4::ResponseType::Ok {
                                if let Err(e) = std::fs::remove_file(&p2) {
                                    let alert = gtk4::MessageDialog::builder()
                                        .transient_for(&w_del)
                                        .modal(true)
                                        .message_type(gtk4::MessageType::Error)
                                        .buttons(gtk4::ButtonsType::Ok)
                                        .text("Failed to Delete Key")
                                        .secondary_text(format!("Could not delete private key: {}", e))
                                        .build();
                                    alert.connect_response(|a, _| a.close());
                                    alert.present();
                                    d.close();
                                    return;
                                }
                                if let Err(e) = std::fs::remove_file(&p1) {
                                    let alert = gtk4::MessageDialog::builder()
                                        .transient_for(&w_del)
                                        .modal(true)
                                        .message_type(gtk4::MessageType::Error)
                                        .buttons(gtk4::ButtonsType::Ok)
                                        .text("Failed to Delete Key")
                                        .secondary_text(format!("Private key deleted, but could not delete public key: {}", e))
                                        .build();
                                    alert.connect_response(|a, _| a.close());
                                    alert.present();
                                    d.close();
                                    return;
                                }
                                if let Some(rc) = h.upgrade()
                                    && let Some(r) = rc.borrow().as_ref() { r(); }
                            }
                            d.close();
                        });
                        dialog.present();
                    });

                    deploy_btn.connect_clicked(move |_| {
                        show_deploy_dialog(&w_clone2, &key_clone2);
                    });

                    hbox.append(&icon);
                    hbox.append(&name_lbl);
                    hbox.append(&deploy_btn);
                    hbox.append(&del_btn);
                    row.set_child(Some(&hbox));
                    lb.append(&row);
                }
            }
        })
    };

    *refresh_ui.borrow_mut() = Some(do_refresh.clone());
    do_refresh();

    let r_refresh = do_refresh.clone();
    refresh_btn.connect_clicked(move |_| { r_refresh(); });

    let r_win = window_rc.clone();
    let g_refresh = do_refresh.clone();
    gen_btn.connect_clicked(move |_| {
        show_generate_dialog(&r_win, g_refresh.clone());
    });
    let w_win = window_rc.clone();
    let i_refresh = do_refresh.clone();
    import_btn.connect_clicked(move |_| {
        show_import_dialog(&w_win, i_refresh.clone());
    });

    main_box
}

fn show_deploy_dialog(parent: &gtk4::ApplicationWindow, key: &SshKeyPair) {
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title(format!("Deploy key: {}", key.name))
        .default_width(350)
        .build();

    let content = dialog.content_area();
    content.set_margin_top(12); content.set_margin_bottom(12);
    content.set_margin_start(12); content.set_margin_end(12);
    content.set_spacing(12);

    let hosts = load_hosts();
    if hosts.is_empty() {
        content.append(&gtk4::Label::new(Some("No servers available.")));
        dialog.add_button("Close", gtk4::ResponseType::Close);
        dialog.connect_response(|d, _| d.close());
        dialog.present();
        return;
    }

    let model = gtk4::StringList::new(&[]);
    for h in &hosts {
        model.append(&h.alias);
    }

    let dropdown = gtk4::DropDown::new(Some(model), gtk4::Expression::NONE);
    content.append(&gtk4::Label::builder().label("Select Server").halign(gtk4::Align::Start).build());
    content.append(&dropdown);

    let pass_entry = gtk4::PasswordEntry::builder()
        .placeholder_text("Server Password (optional if agent is running)")
        .show_peek_icon(true)
        .build();
    content.append(&gtk4::Label::builder().label("Password (for deployment)").halign(gtk4::Align::Start).build());
    content.append(&pass_entry);

    let status_label = gtk4::Label::new(None);
    status_label.set_halign(gtk4::Align::Start);
    content.append(&status_label);

    let _ok_btn = dialog.add_button("Deploy", gtk4::ResponseType::Ok);
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);

    let key_path = key.pub_path.clone();
    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Ok {
            let idx = dropdown.selected();
            if idx < hosts.len() as u32 {
                let host = hosts[idx as usize].clone();
                let password = pass_entry.text().to_string();
                let pubkey = match std::fs::read_to_string(&key_path) {
                    Ok(content) => content,
                    Err(e) => {
                        let md = gtk4::MessageDialog::builder()
                            .modal(true)
                            .message_type(gtk4::MessageType::Error)
                            .buttons(gtk4::ButtonsType::Ok)
                            .text("Failed to Read Public Key")
                            .secondary_text(format!("Could not read '{}': {}", key_path.display(), e))
                            .build();
                        if let Some(w) = d.transient_for() {
                            md.set_transient_for(Some(&w));
                        }
                        md.connect_response(|md, _| md.close());
                        md.present();
                        return;
                    }
                };
                let parent_win_weak = d.transient_for().and_then(|w| w.downcast::<gtk4::Window>().ok());
                let close_dialog = d.clone();
                glib::MainContext::default().spawn_local(async move {
                    let mut final_password = None;
                    if !password.is_empty() {
                        final_password = Some(password);
                    } else if let Ok(keyring) = oo7::Keyring::new().await {
                        let mut attr = std::collections::HashMap::new();
                        let alias_lower = host.alias.to_lowercase();
                        attr.insert("rustmius-server-alias", alias_lower.as_str());
                        if let Ok(items) = keyring.search_items(&attr).await
                            && let Some(item) = items.first()
                                && let Ok(pass) = item.secret().await {
                                    final_password = std::str::from_utf8(pass.as_ref()).map(String::from).ok();
                                }
                    }

                    let h_c = host.clone();
                    let pk_c = pubkey.clone();
                    let result = tokio::task::spawn_blocking(move || {
                        crate::ssh_engine::deploy_pubkey(&h_c, final_password, &pk_c)
                    }).await.unwrap_or_else(|_| Err(anyhow::anyhow!("Task panic")));

                    match result {
                        Ok(_) => {
                            let md = gtk4::MessageDialog::builder()
                                .modal(true)
                                .message_type(gtk4::MessageType::Info)
                                .buttons(gtk4::ButtonsType::Ok)
                                .text("Deployed Successfully!")
                                .build();
                            if let Some(ref w) = parent_win_weak {
                                md.set_transient_for(Some(w));
                            }
                            md.connect_response(|md, _| md.close());
                            md.present();
                            close_dialog.close();
                        },
                        Err(e) => {
                            let md = gtk4::MessageDialog::builder()
                                .modal(true)
                                .message_type(gtk4::MessageType::Error)
                                .buttons(gtk4::ButtonsType::Ok)
                                .text("Deployment Failed")
                                .secondary_text(e.to_string())
                                .build();
                            if let Some(ref w) = parent_win_weak {
                                md.set_transient_for(Some(w));
                            }
                            md.connect_response(|md, _| md.close());
                            md.present();
                        }
                    }
                });
            }
        } else {
            d.close();
        }
    });

    dialog.present();
}

fn show_generate_dialog(parent: &gtk4::ApplicationWindow, on_save: Rc<dyn Fn()>) {
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Generate SSH Key")
        .default_width(350)
        .build();

    let content = dialog.content_area();
    content.set_margin_top(12); content.set_margin_bottom(12);
    content.set_margin_start(12); content.set_margin_end(12);
    content.set_spacing(12);

    let name_entry = gtk4::Entry::builder().placeholder_text("Key Name (e.g. id_ed25519_mykey)").build();
    let pass_entry = gtk4::PasswordEntry::builder().placeholder_text("Passphrase (optional)").show_peek_icon(true).build();
    let comment_entry = gtk4::Entry::builder().placeholder_text("Comment (optional, e.g. user@hostname)").build();

    content.append(&gtk4::Label::builder().label("Key Filename").halign(gtk4::Align::Start).build());
    content.append(&name_entry);
    content.append(&gtk4::Label::builder().label("Passphrase").halign(gtk4::Align::Start).build());
    content.append(&pass_entry);
    content.append(&gtk4::Label::builder().label("Comment").halign(gtk4::Align::Start).build());
    content.append(&comment_entry);

    let ok_btn = dialog.add_button("Generate", gtk4::ResponseType::Ok);
    ok_btn.set_sensitive(false);
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);

    let ok_rc = ok_btn.clone();
    name_entry.connect_changed(move |e| {
        ok_rc.set_sensitive(!e.text().is_empty());
    });

    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Ok {
            let name = name_entry.text().to_string();
            let pass = pass_entry.text().to_string();
            let comment = comment_entry.text().to_string();

            if !is_valid_key_name(&name) {
                let alert = make_error_alert(
                    d.transient_for().as_ref().map(|w| w.upcast_ref()),
                    "Invalid Key Name",
                    "The key name must be a simple filename with no path separators or special components.",
                );
                alert.present();
                return;
            }
            if let Some(ssh_dir) = get_ssh_dir() {
                let file_path = ssh_dir.join(&name);
                let pub_path = ssh_dir.join(format!("{}.pub", name));

                if file_path.exists() || pub_path.exists() {
                    let alert = make_error_alert(
                        d.transient_for().as_ref().map(|w| w.upcast_ref()),
                        "Key Already Exists",
                        &format!("A file named '{}' or its public key already exists in ~/.ssh/. Choose a different name.", name),
                    );
                    alert.present();
                    return;
                }

                let parent_win = d.transient_for()
                    .and_then(|w| w.downcast::<gtk4::ApplicationWindow>().ok());
                d.close();

                let on_save_spawn = on_save.clone();
                glib::MainContext::default().spawn_local(async move {
                    let result = tokio::task::spawn_blocking(move || {
                        let mut cmd = std::process::Command::new("ssh-keygen");
                        cmd.arg("-t").arg("ed25519")
                           .arg("-f").arg(&file_path)
                           .arg("-N").arg(&pass)
                           .arg("-q");
                        if !comment.is_empty() {
                            cmd.arg("-C").arg(&comment);
                        }
                        cmd.output()
                    }).await;

                    let (success, stderr_msg) = match result {
                        Ok(Ok(output)) => (output.status.success(), String::from_utf8_lossy(&output.stderr).to_string()),
                        Ok(Err(e)) => (false, e.to_string()),
                        Err(e) => (false, e.to_string()),
                    };

                    if success {
                        on_save_spawn();
                    } else {
                        let secondary = if stderr_msg.is_empty() {
                            "ssh-keygen exited with a non-zero status.".to_string()
                        } else {
                            stderr_msg
                        };
                        let alert = make_error_alert(
                            parent_win.as_ref().map(|w| w.upcast_ref()),
                            "Key Generation Failed!",
                            &secondary,
                        );
                        alert.present();
                    }
                });
            }
        } else {
            d.close();
        }
    });

    dialog.present();
}

fn show_import_dialog(parent: &gtk4::ApplicationWindow, on_save: Rc<dyn Fn()>) {
    let dialog = gtk4::Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Import Private Key")
        .default_width(450)
        .default_height(400)
        .build();

    let content = dialog.content_area();
    content.set_margin_top(12); content.set_margin_bottom(12);
    content.set_margin_start(12); content.set_margin_end(12);
    content.set_spacing(12);

    let name_entry = gtk4::Entry::builder().placeholder_text("Key Name (e.g. id_rsa)").build();
    let text_buffer = gtk4::TextBuffer::new(None);
    let text_view = gtk4::TextView::builder()
        .buffer(&text_buffer)
        .monospace(true)
        .vexpand(true)
        .build();
    let scrolled = gtk4::ScrolledWindow::builder()
        .child(&text_view)
        .min_content_height(250)
        .vexpand(true)
        .build();

    content.append(&gtk4::Label::builder().label("Key Filename").halign(gtk4::Align::Start).build());
    content.append(&name_entry);
    content.append(&gtk4::Label::builder().label("Paste Private Key").halign(gtk4::Align::Start).build());
    content.append(&scrolled);

    let ok_btn = dialog.add_button("Import", gtk4::ResponseType::Ok);
    ok_btn.set_sensitive(false);
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);

    let ok_rc = ok_btn.clone();
    name_entry.connect_changed(move |e| {
        ok_rc.set_sensitive(!e.text().is_empty());
    });

    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Ok {
            let name = name_entry.text().to_string();
            let (start, end) = text_buffer.bounds();
            let key_content = text_buffer.text(&start, &end, false).to_string();

            if !is_valid_key_name(&name) {
                let alert = make_error_alert(
                    d.transient_for().as_ref().map(|w| w.upcast_ref()),
                    "Invalid Key Name",
                    "The key name must be a simple filename with no path separators or special components.",
                );
                alert.present();
                return;
            }
            if let Some(ssh_dir) = get_ssh_dir() {
                let file_path = ssh_dir.join(&name);
                let pub_path = ssh_dir.join(format!("{}.pub", name));

                if file_path.exists() || pub_path.exists() {
                    let alert = make_error_alert(
                        d.transient_for().as_ref().map(|w| w.upcast_ref()),
                        "Key Already Exists",
                        &format!("A file named '{}' or its public key already exists in ~/.ssh/.", name),
                    );
                    alert.present();
                    return;
                }

                #[cfg(unix)]
                let write_result = {
                    use std::os::unix::fs::OpenOptionsExt;
                    std::fs::OpenOptions::new()
                        .create(true)
                        .truncate(true)
                        .write(true)
                        .mode(0o600)
                        .open(&file_path)
                        .and_then(|mut f| {
                            use std::io::Write;
                            f.write_all(key_content.as_bytes())
                        })
                };
                #[cfg(not(unix))]
                let write_result = std::fs::write(&file_path, &key_content);

                if let Err(e) = write_result {
                    let alert = make_error_alert(
                        d.transient_for().as_ref().map(|w| w.upcast_ref()),
                        "Failed to Write Key File",
                        &e.to_string(),
                    );
                    alert.present();
                    return;
                }

                let parent_win = d.transient_for()
                    .and_then(|w| w.downcast::<gtk4::ApplicationWindow>().ok());
                d.close();

                let on_save_spawn = on_save.clone();
                glib::MainContext::default().spawn_local(async move {
                    let pub_path = ssh_dir.join(format!("{}.pub", name));
                    let file_path_cleanup = ssh_dir.join(&name);
                    let file_path_keygen = ssh_dir.join(&name);
                    let result = tokio::task::spawn_blocking(move || {
                        std::process::Command::new("ssh-keygen")
                            .arg("-y")
                            .arg("-f").arg(&file_path_keygen)
                            .output()
                    }).await;

                    match result {
                        Ok(Ok(output)) if output.status.success() => {
                            if let Err(e) = std::fs::write(&pub_path, output.stdout) {
                                let _ = std::fs::remove_file(&file_path_cleanup);
                                let alert = make_error_alert(
                                    parent_win.as_ref().map(|w| w.upcast_ref()),
                                    "Failed to Write Public Key",
                                    &e.to_string(),
                                );
                                alert.present();
                            } else {
                                on_save_spawn();
                            }
                        }
                        Ok(Ok(output)) => {
                            let _ = std::fs::remove_file(&file_path_cleanup);
                            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                            let secondary = if stderr.is_empty() {
                                "Check if the pasted key is a valid private key or if it is encrypted.".to_string()
                            } else {
                                stderr
                            };
                            let alert = make_error_alert(
                                parent_win.as_ref().map(|w| w.upcast_ref()),
                                "Key Import Failed!",
                                &secondary,
                            );
                            alert.present();
                        }
                        Ok(Err(e)) => {
                            let _ = std::fs::remove_file(&file_path_cleanup);
                            let alert = make_error_alert(
                                parent_win.as_ref().map(|w| w.upcast_ref()),
                                "Key Import Failed!",
                                &e.to_string(),
                            );
                            alert.present();
                        }
                        Err(e) => {
                            let _ = std::fs::remove_file(&file_path_cleanup);
                            let alert = make_error_alert(
                                parent_win.as_ref().map(|w| w.upcast_ref()),
                                "Key Import Failed!",
                                &e.to_string(),
                            );
                            alert.present();
                        }
                    }
                });
            }
        } else {
            d.close();
        }
    });

    dialog.present();
}