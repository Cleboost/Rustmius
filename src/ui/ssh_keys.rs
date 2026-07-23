#![allow(deprecated)]
use crate::config_observer::{REMOTE_SSH_DIR, SshKeyPair, get_ssh_dir, load_hosts, load_ssh_keys};
use gtk4::prelude::*;
use gtk4::{gio, glib};
use std::cell::RefCell;
use std::rc::Rc;

fn is_valid_key_name(name: &str) -> bool {
    if name.is_empty() || name.contains('\0') {
        return false;
    }
    let p = std::path::Path::new(name);
    p.components().count() == 1
        && p.file_name()
            .map(|n| n == std::ffi::OsStr::new(name))
            .unwrap_or(false)
}

fn show_error_alert(parent: Option<&gtk4::Window>, title: &str, secondary: &str) {
    let dialog = gtk4::AlertDialog::builder()
        .modal(true)
        .message(title)
        .detail(secondary)
        .buttons(vec!["OK"])
        .default_button(0)
        .build();
    dialog.show(parent);
}

pub fn build_ssh_keys_ui(window: &gtk4::ApplicationWindow) -> gtk4::Box {
    let main_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    main_box.add_css_class("page");

    let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
    header_box.add_css_class("page-header");
    let title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
    title_box.set_hexpand(true);
    let title = gtk4::Label::builder()
        .label("SSH Keys")
        .halign(gtk4::Align::Start)
        .build();
    title.add_css_class("title-1");
    let subtitle = gtk4::Label::builder()
        .label("Manage your SSH key pairs")
        .halign(gtk4::Align::Start)
        .css_classes(vec!["page-subtitle".to_string()])
        .build();
    title_box.append(&title);
    title_box.append(&subtitle);

    let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    actions_box.add_css_class("page-header-actions");
    let gen_btn = gtk4::Button::from_icon_name("list-add-symbolic");
    gen_btn.set_tooltip_text(Some("Generate New Key"));
    gen_btn.add_css_class("suggested-action");
    let import_btn = gtk4::Button::from_icon_name("document-import-symbolic");
    import_btn.set_tooltip_text(Some("Import Key"));
    import_btn.add_css_class("flat");
    let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
    refresh_btn.set_tooltip_text(Some("Refresh"));
    refresh_btn.add_css_class("flat");

    actions_box.append(&refresh_btn);
    actions_box.append(&import_btn);
    actions_box.append(&gen_btn);

    header_box.append(&title_box);
    header_box.append(&actions_box);
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

            let keys = load_ssh_keys().unwrap_or_else(|e| {
                tracing::error!("Failed to load SSH keys: {}", e);
                Vec::new()
            });
            if keys.is_empty() {
                let empty_lbl =
                    gtk4::Label::new(Some(&format!("No SSH keys found in {}/", REMOTE_SSH_DIR)));
                empty_lbl.set_margin_top(24);
                empty_lbl.set_margin_bottom(24);
                empty_lbl.add_css_class("dim-label");
                lb.append(&empty_lbl);
            } else {
                for key in keys {
                    let row = gtk4::ListBoxRow::new();
                    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 14);
                    hbox.add_css_class("list-row-content");
                    let icon = gtk4::Image::from_icon_name("network-vpn-symbolic");
                    icon.set_pixel_size(20);
                    icon.set_opacity(0.7);
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
                        let dialog = gtk4::AlertDialog::builder()
                            .modal(true)
                            .message(format!("Delete key '{}'?", key_clone1.name))
                            .detail("This action cannot be undone and will delete both public and private key files.")
                            .buttons(vec!["Cancel", "Delete"])
                            .cancel_button(0)
                            .default_button(1)
                            .build();

                        let p1 = key_clone1.pub_path.clone();
                        let p2 = key_clone1.priv_path.clone();
                        let h = handle.clone();
                        let w_del = w_clone.clone();

                        dialog.choose(Some(&w_clone), None::<&gio::Cancellable>, move |res| {
                            if let Ok(idx) = res
                                && idx == 1 {
                                    if let Err(e) = std::fs::remove_file(&p2) {
                                        show_error_alert(Some(w_del.upcast_ref::<gtk4::Window>()), "Failed to Delete Key", &format!("Could not delete private key: {}", e));
                                        return;
                                    }
                                    if let Err(e) = std::fs::remove_file(&p1) {
                                        show_error_alert(Some(w_del.upcast_ref::<gtk4::Window>()), "Failed to Delete Key", &format!("Private key deleted, but could not delete public key: {}", e));
                                        return;
                                    }
                                    if let Some(rc) = h.upgrade()
                                        && let Some(r) = rc.borrow().as_ref() { r(); }
                                }
                        });
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
    refresh_btn.connect_clicked(move |_| {
        r_refresh();
    });

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
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_spacing(12);

    let hosts = load_hosts().unwrap_or_else(|e| {
        tracing::error!("Failed to load hosts: {}", e);
        Vec::new()
    });
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
    content.append(
        &gtk4::Label::builder()
            .label("Select Server")
            .halign(gtk4::Align::Start)
            .build(),
    );
    content.append(&dropdown);

    let pass_entry = gtk4::PasswordEntry::builder()
        .placeholder_text("Server Password (optional if agent is running)")
        .show_peek_icon(true)
        .build();
    content.append(
        &gtk4::Label::builder()
            .label("Password (for deployment)")
            .halign(gtk4::Align::Start)
            .build(),
    );
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
                        show_error_alert(
                            d.transient_for().as_ref().map(|w| w.upcast_ref()),
                            "Failed to Read Public Key",
                            &format!("Could not read '{}': {}", key_path.display(), e),
                        );
                        return;
                    }
                };
                let parent_win_weak = d
                    .transient_for()
                    .and_then(|w| w.downcast::<gtk4::Window>().ok());
                let close_dialog = d.clone();
                glib::MainContext::default().spawn_local(async move {
                    let final_password = if !password.is_empty() {
                        Some(password)
                    } else {
                        crate::config_observer::get_keyring_password(&host.alias).await
                    };

                    let result = crate::engines::ssh::deploy_pubkey(
                        &host,
                        final_password.as_deref(),
                        &pubkey,
                    )
                    .await;

                    match result {
                        Ok(_) => {
                            let md = gtk4::AlertDialog::builder()
                                .modal(true)
                                .message("Deployed Successfully!")
                                .buttons(vec!["OK"])
                                .default_button(0)
                                .build();
                            md.show(
                                parent_win_weak
                                    .as_ref()
                                    .map(|w| w.upcast_ref::<gtk4::Window>()),
                            );
                            close_dialog.close();
                        }
                        Err(e) => {
                            show_error_alert(
                                parent_win_weak
                                    .as_ref()
                                    .map(|w| w.upcast_ref::<gtk4::Window>()),
                                "Deployment Failed",
                                &e.to_string(),
                            );
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
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_spacing(12);

    let name_entry = gtk4::Entry::builder()
        .placeholder_text("Key Name (e.g. id_ed25519_mykey)")
        .build();
    let pass_entry = gtk4::PasswordEntry::builder()
        .placeholder_text("Passphrase (optional)")
        .show_peek_icon(true)
        .build();
    let comment_entry = gtk4::Entry::builder()
        .placeholder_text("Comment (optional, e.g. user@hostname)")
        .build();

    content.append(
        &gtk4::Label::builder()
            .label("Key Filename")
            .halign(gtk4::Align::Start)
            .build(),
    );
    content.append(&name_entry);
    content.append(
        &gtk4::Label::builder()
            .label("Passphrase")
            .halign(gtk4::Align::Start)
            .build(),
    );
    content.append(&pass_entry);
    content.append(
        &gtk4::Label::builder()
            .label("Comment")
            .halign(gtk4::Align::Start)
            .build(),
    );
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
                show_error_alert(
                    d.transient_for().as_ref().map(|w| w.upcast_ref()),
                    "Invalid Key Name",
                    "The key name must be a simple filename with no path separators or special components.",
                );
                return;
            }
            if let Some(ssh_dir) = get_ssh_dir() {
                let file_path = ssh_dir.join(&name);
                let pub_path = ssh_dir.join(format!("{}.pub", name));

                if file_path.exists() || pub_path.exists() {
                    show_error_alert(
                        d.transient_for().as_ref().map(|w| w.upcast_ref()),
                        "Key Already Exists",
                        &format!("A file named '{}' or its public key already exists in {}. Choose a different name.", name, REMOTE_SSH_DIR),
                    );
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
                        show_error_alert(
                            parent_win.as_ref().map(|w| w.upcast_ref()),
                            "Key Generation Failed!",
                            &secondary,
                        );
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
    content.set_margin_top(12);
    content.set_margin_bottom(12);
    content.set_margin_start(12);
    content.set_margin_end(12);
    content.set_spacing(12);

    let name_entry = gtk4::Entry::builder()
        .placeholder_text("Key Name (e.g. id_rsa)")
        .build();
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

    content.append(
        &gtk4::Label::builder()
            .label("Key Filename")
            .halign(gtk4::Align::Start)
            .build(),
    );
    content.append(&name_entry);
    content.append(
        &gtk4::Label::builder()
            .label("Paste Private Key")
            .halign(gtk4::Align::Start)
            .build(),
    );
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
            // Private key material: hold it in a buffer that is zeroed on drop.
            let key_content = zeroize::Zeroizing::new(text_buffer.text(&start, &end, false).to_string());

            if !is_valid_key_name(&name) {
                show_error_alert(
                    d.transient_for().as_ref().map(|w| w.upcast_ref()),
                    "Invalid Key Name",
                    "The key name must be a simple filename with no path separators or special components.",
                );
                return;
            }
            if let Some(ssh_dir) = get_ssh_dir() {
                let file_path = ssh_dir.join(&name);
                let pub_path = ssh_dir.join(format!("{}.pub", name));

                if file_path.exists() || pub_path.exists() {
                    show_error_alert(
                        d.transient_for().as_ref().map(|w| w.upcast_ref()),
                        "Key Already Exists",
                        &format!("A file named '{}' or its public key already exists in {}.", name, REMOTE_SSH_DIR),
                    );
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
                let write_result = std::fs::write(&file_path, key_content.as_bytes());

                if let Err(e) = write_result {
                    show_error_alert(
                        d.transient_for().as_ref().map(|w| w.upcast_ref()),
                        "Failed to Write Key File",
                        &e.to_string(),
                    );
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
                                show_error_alert(
                                    parent_win.as_ref().map(|w| w.upcast_ref()),
                                    "Failed to Write Public Key",
                                    &e.to_string(),
                                );
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
                            show_error_alert(
                                parent_win.as_ref().map(|w| w.upcast_ref()),
                                "Key Import Failed!",
                                &secondary,
                            );
                        }
                        Ok(Err(e)) => {
                            let _ = std::fs::remove_file(&file_path_cleanup);
                            show_error_alert(
                                parent_win.as_ref().map(|w| w.upcast_ref()),
                                "Key Import Failed!",
                                &e.to_string(),
                            );
                        }
                        Err(e) => {
                            let _ = std::fs::remove_file(&file_path_cleanup);
                            show_error_alert(
                                parent_win.as_ref().map(|w| w.upcast_ref()),
                                "Key Import Failed!",
                                &e.to_string(),
                            );
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
