use gtk4::glib::{self, ControlFlow};
use gtk4::{prelude::*, Box, Button, Dialog, Entry, HeaderBar, Label, Orientation, Spinner};
use std::process::Command;
use std::sync::mpsc::{channel, TryRecvError};
use std::thread;

pub fn create_export_key_dialog(
    parent: &Dialog,
    user: &str,
    host: &str,
    pub_key_path: &str,
) -> Dialog {
    let dialog = Dialog::builder()
        .transient_for(parent)
        .modal(true)
        .title("Export SSH Key")
        .build();

    let header = HeaderBar::new();
    header.set_title_widget(Some(&Label::new(Some("Export SSH Key"))));
    dialog.set_titlebar(Some(&header));

    let vbox = Box::new(Orientation::Vertical, 12);
    vbox.set_margin_top(24);
    vbox.set_margin_bottom(24);
    vbox.set_margin_start(24);
    vbox.set_margin_end(24);

    let pass_entry = Entry::builder()
        .placeholder_text("Password")
        .visibility(false)
        .primary_icon_name("dialog-password-symbolic")
        .build();
    vbox.append(&pass_entry);

    let status_label = Label::new(Some(""));
    status_label.set_visible(false);
    vbox.append(&status_label);

    let spinner = Spinner::new();
    spinner.set_visible(false);
    vbox.append(&spinner);

    let cancel_btn = Button::with_label("Cancel");
    let export_btn = Button::with_label("Export");
    export_btn.add_css_class("suggested-action");

    let hbox = Box::new(Orientation::Horizontal, 12);
    hbox.set_halign(gtk4::Align::End);
    hbox.append(&cancel_btn);
    hbox.append(&export_btn);
    vbox.append(&hbox);

    dialog.set_child(Some(&vbox));

    let dialog_clone1 = dialog.clone();
    cancel_btn.connect_clicked(move |_| {
        dialog_clone1.close();
    });
    let user = user.to_string();
    let host = host.to_string();
    let pub_key_path = pub_key_path.to_string();
    let pass_entry_clone = pass_entry.clone();
    let export_btn_clone = export_btn.clone();
    let cancel_btn_clone = cancel_btn.clone();
    let status_label_clone = status_label.clone();
    let spinner_clone = spinner.clone();
    let dialog_final = dialog.clone();

    export_btn.connect_clicked(move |_| {
        let password = pass_entry_clone.text().to_string();
        if password.is_empty() {
            return;
        }

        pass_entry_clone.set_sensitive(false);
        export_btn_clone.set_sensitive(false);
        cancel_btn_clone.set_sensitive(false);
        status_label_clone.set_text("Exporting SSH key...");
        status_label_clone.set_visible(true);
        spinner_clone.start();
        spinner_clone.set_visible(true);

        let (tx, rx) = channel::<Result<(), String>>();
        let pub_key_path_bg = pub_key_path.clone();
        let user_bg = user.clone();
        let host_bg = host.clone();
        thread::spawn(move || {
            let status = Command::new("sshpass")
                .args([
                    "-p",
                    &password,
                    "ssh-copy-id",
                    "-i",
                    &pub_key_path_bg,
                    &format!("{}@{}", user_bg, host_bg),
                ])
                .status();

            let result = match status {
                Ok(s) if s.success() => Ok(()),
                Ok(_s) => Err("Invalid credentials or connection failed".to_string()),
                Err(e) => Err(format!("Connection error - {}", e)),
            };

            let _ = tx.send(result);
        });

        let status_label_ui = status_label_clone.clone();
        let spinner_ui = spinner_clone.clone();
        let export_btn_ui = export_btn_clone.clone();
        let dialog_ui = dialog_final.clone();
        glib::idle_add_local(move || match rx.try_recv() {
            Ok(result) => {
                spinner_ui.stop();
                spinner_ui.set_visible(false);

                match result {
                    Ok(()) => {
                        status_label_ui.set_text("✓ SSH key exported successfully!");
                        status_label_ui.add_css_class("success");
                    }
                    Err(msg) => {
                        status_label_ui.set_text(&format!("✗ {}", msg));
                        status_label_ui.add_css_class("error");
                    }
                }

                export_btn_ui.set_label("Close");
                export_btn_ui.set_sensitive(true);

                let dialog_close = dialog_ui.clone();
                export_btn_ui.connect_clicked(move |_| {
                    dialog_close.close();
                });

                ControlFlow::Break
            }
            Err(TryRecvError::Empty) => ControlFlow::Continue,
            Err(TryRecvError::Disconnected) => {
                spinner_ui.stop();
                spinner_ui.set_visible(false);
                status_label_ui.set_text("✗ Internal error - worker disconnected");
                status_label_ui.add_css_class("error");
                export_btn_ui.set_label("Close");
                export_btn_ui.set_sensitive(true);

                let dialog_close = dialog_ui.clone();
                export_btn_ui.connect_clicked(move |_| {
                    dialog_close.close();
                });

                ControlFlow::Break
            }
        });
    });

    dialog
}
