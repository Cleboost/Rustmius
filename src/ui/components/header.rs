use gtk4::prelude::*;

pub struct Header {
    pub container: gtk4::HeaderBar,
    pub add_btn: gtk4::Button,
}

impl Header {
    pub fn new() -> Self {
        let container = gtk4::HeaderBar::new();
        let add_btn = gtk4::Button::from_icon_name("list-add-symbolic");
        add_btn.add_css_class("suggested-action");
        container.pack_start(&add_btn);

        Self {
            container,
            add_btn,
        }
    }
}
