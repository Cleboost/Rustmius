use gtk4::prelude::*;

pub struct Header {
    pub container: gtk4::HeaderBar,
    pub add_btn: gtk4::Button,
}

impl Header {
    pub fn new() -> Self {
        let container = gtk4::HeaderBar::new();
        container.add_css_class("main-headerbar");
        
        let add_btn = gtk4::Button::from_icon_name("list-add-symbolic");
        add_btn.add_css_class("suggested-action");
        add_btn.set_margin_start(11);
        add_btn.set_valign(gtk4::Align::Center);
        
        container.pack_start(&add_btn);

        Self {
            container,
            add_btn,
        }
    }
}
