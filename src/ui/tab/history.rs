use crate::service::load_history;
use chrono::{DateTime, Local};
use gtk4::prelude::*;
use gtk4::{Box, Label, ListBox, ListBoxRow, Orientation, ScrolledWindow};
use std::rc::Rc;

fn build_list() -> ListBox {
    let list = ListBox::new();
    list.set_selection_mode(gtk4::SelectionMode::None);

    let entries = load_history();
    for (i, entry) in entries.iter().rev().enumerate() {
        let row = ListBoxRow::new();

        row.add_css_class(&format!("history-row-{}", i));

        let row_box = Box::new(Orientation::Horizontal, 12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

        let connection_icon = gtk4::Image::from_icon_name("network-wired-symbolic");
        connection_icon.set_icon_size(gtk4::IconSize::Normal);
        connection_icon.set_margin_end(8);
        row_box.append(&connection_icon);

        let text_container = Box::new(Orientation::Vertical, 4);
        text_container.set_hexpand(true);

        let name_label = Label::new(Some(&entry.server_name));
        name_label.add_css_class("title-4");
        name_label.set_halign(gtk4::Align::Start);

        let when = match entry.timestamp.parse::<DateTime<Local>>() {
            Ok(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
            Err(_) => entry.timestamp.clone(),
        };
        let ts_label = Label::new(Some(&when));
        ts_label.add_css_class("dim-label");
        ts_label.set_halign(gtk4::Align::Start);

        text_container.append(&name_label);
        text_container.append(&ts_label);
        row_box.append(&text_container);
        row.set_child(Some(&row_box));
        list.append(&row);
    }
    list
}

pub fn create_history_tab() -> (Box, Rc<dyn Fn()>) {
    let container = Box::new(Orientation::Vertical, 0);

    let header_container = Box::new(Orientation::Vertical, 20);
    header_container.set_margin_top(20);
    header_container.set_margin_bottom(20);
    header_container.set_margin_start(12);
    header_container.set_margin_end(12);
    header_container.set_halign(gtk4::Align::Fill);
    header_container.set_hexpand(true);

    let title = Label::new(Some("History"));
    title.add_css_class("title-1");
    title.set_halign(gtk4::Align::Start);
    header_container.append(&title);

    let description = Label::new(Some("Historique des connexions SSH (à venir)"));
    description.add_css_class("dim-label");
    description.set_halign(gtk4::Align::Start);
    header_container.append(&description);

    container.append(&header_container);

    let scrolled = ScrolledWindow::builder()
        .hscrollbar_policy(gtk4::PolicyType::Automatic)
        .vscrollbar_policy(gtk4::PolicyType::Automatic)
        .hexpand(true)
        .vexpand(true)
        .build();
    let list = build_list();
    scrolled.set_child(Some(&list));
    container.append(&scrolled);

    let list_scrolled = scrolled.clone();
    let refresh_fn: Rc<dyn Fn()> = Rc::new(move || {
        if let Some(old_child) = list_scrolled.child() {
            list_scrolled.set_child(None::<&gtk4::Widget>);
            let _ = old_child;
        }
        let new_list = build_list();
        list_scrolled.set_child(Some(&new_list));
    });

    (container, refresh_fn)
}
