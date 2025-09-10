use gtk4::gdk::Display;
use gtk4::prelude::*;
use gtk4::{Box, Button, Label, Orientation, PolicyType, ScrolledWindow, TextBuffer, TextView};
use libadwaita::prelude::AdwDialogExt;
use libadwaita::{Dialog, Toast, ToastOverlay};
use std::cell::RefCell;
use std::rc::Rc;

pub fn create_info_key_dialog(
    key_name: &str,
    key_type: &str,
    public_content: Option<&str>,
    private_content: Option<&str>,
    toast_overlay: Rc<ToastOverlay>,
) -> Dialog {
    let dialog = Dialog::builder().title("SSH Key Information").build();

    let close_button = Button::builder()
        .icon_name("window-close-symbolic")
        .tooltip_text("Close")
        .css_classes(vec!["circular", "flat"])
        .build();

    close_button.connect_clicked({
        let dialog = dialog.clone();
        move |_| {
            dialog.close();
        }
    });

    let content_box = Box::new(Orientation::Vertical, 12);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let header_box = Box::new(Orientation::Horizontal, 0);
    header_box.set_hexpand(true);

    let title_label = Label::new(Some(&format!("Key: {} ({})", key_name, key_type)));
    title_label.add_css_class("title-2");
    title_label.set_halign(gtk4::Align::Start);
    title_label.set_hexpand(true);

    header_box.append(&title_label);
    header_box.append(&close_button);
    content_box.append(&header_box);

    let pub_header_box = Box::new(Orientation::Horizontal, 0);
    pub_header_box.set_hexpand(true);

    let pub_label = Label::new(Some("Public key:"));
    pub_label.set_halign(gtk4::Align::Start);
    pub_label.set_hexpand(true);

    let pub_clipboard_button = Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy public key")
        .css_classes(vec!["circular", "flat"])
        .build();

    pub_header_box.append(&pub_label);
    pub_header_box.append(&pub_clipboard_button);
    content_box.append(&pub_header_box);

    let pub_buffer = TextBuffer::new(None);
    let pub_text = public_content.unwrap_or("Public key not available");
    pub_buffer.set_text(pub_text);
    let pub_view = TextView::new();
    pub_view.set_buffer(Some(&pub_buffer));
    pub_view.set_editable(false);
    pub_view.set_cursor_visible(true);
    pub_view.set_monospace(true);
    pub_view.set_wrap_mode(gtk4::WrapMode::Char);
    pub_view.set_vexpand(false);
    pub_view.set_hexpand(true);
    pub_view.set_height_request(120);

    let pub_scroll = ScrolledWindow::new();
    pub_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    pub_scroll.set_child(Some(&pub_view));
    pub_scroll.set_min_content_height(120);
    content_box.append(&pub_scroll);

    pub_clipboard_button.connect_clicked({
        let pub_buffer = pub_buffer.clone();
        let toast_overlay = toast_overlay.clone();
        move |_| {
            let text = pub_buffer.text(&pub_buffer.start_iter(), &pub_buffer.end_iter(), false);
            if let Some(display) = Display::default() {
                let clipboard = display.clipboard();
                clipboard.set_text(&text.to_string());

                let toast = Toast::new("Public key copied to clipboard");
                toast_overlay.add_toast(toast);
            }
        }
    });

    let priv_header_box = Box::new(Orientation::Horizontal, 0);
    priv_header_box.set_hexpand(true);

    let priv_label = Label::new(Some("Private key:"));
    priv_label.set_halign(gtk4::Align::Start);
    priv_label.set_hexpand(true);

    let priv_toggle_button = Button::builder()
        .icon_name("eye-not-looking-symbolic")
        .tooltip_text("Show/Hide private key")
        .css_classes(vec!["circular", "flat"])
        .build();

    let priv_clipboard_button = Button::builder()
        .icon_name("edit-copy-symbolic")
        .tooltip_text("Copy private key")
        .css_classes(vec!["circular", "flat"])
        .build();

    priv_header_box.append(&priv_label);
    priv_header_box.append(&priv_toggle_button);
    priv_header_box.append(&priv_clipboard_button);
    content_box.append(&priv_header_box);

    let priv_buffer = TextBuffer::new(None);
    let priv_text = private_content.unwrap_or("Private key not available");
    let hidden_text = "••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••";

    let is_visible = Rc::new(RefCell::new(false));
    let original_content = Rc::new(RefCell::new(priv_text.to_string()));

    priv_buffer.set_text(hidden_text);

    let priv_view = TextView::new();
    priv_view.set_buffer(Some(&priv_buffer));
    priv_view.set_editable(false);
    priv_view.set_cursor_visible(true);
    priv_view.set_monospace(true);
    priv_view.set_wrap_mode(gtk4::WrapMode::Char);
    priv_view.set_vexpand(false);
    priv_view.set_hexpand(true);
    priv_view.set_height_request(120);

    let priv_scroll = ScrolledWindow::new();
    priv_scroll.set_policy(PolicyType::Automatic, PolicyType::Automatic);
    priv_scroll.set_child(Some(&priv_view));
    priv_scroll.set_min_content_height(120);
    content_box.append(&priv_scroll);

    priv_toggle_button.connect_clicked({
        let priv_buffer = priv_buffer.clone();
        let is_visible = is_visible.clone();
        let original_content = original_content.clone();
        let priv_toggle_button = priv_toggle_button.clone();
        move |_| {
            let mut visible = is_visible.borrow_mut();
            *visible = !*visible;

            if *visible {
                priv_buffer.set_text(&original_content.borrow());
                priv_toggle_button.set_icon_name("eye-open-negative-filled-symbolic");
                priv_toggle_button.set_tooltip_text(Some("Hide private key"));
            } else {
                priv_buffer.set_text(hidden_text);
                priv_toggle_button.set_icon_name("eye-not-looking-symbolic");
                priv_toggle_button.set_tooltip_text(Some("Show private key"));
            }
        }
    });

    priv_clipboard_button.connect_clicked({
        let original_content = original_content.clone();
        let toast_overlay = toast_overlay.clone();
        move |_| {
            if let Some(display) = Display::default() {
                let clipboard = display.clipboard();
                clipboard.set_text(&original_content.borrow());

                let toast = Toast::new("Private key copied to clipboard");
                toast_overlay.add_toast(toast);
            }
        }
    });

    dialog.set_child(Some(&content_box));

    dialog
}
