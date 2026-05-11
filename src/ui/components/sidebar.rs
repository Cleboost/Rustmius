use gtk4::prelude::*;

pub struct Sidebar {
    pub container: gtk4::Box,
    pub btn_servers: gtk4::Button,
    pub btn_keys: gtk4::Button,
    pub btn_settings: gtk4::Button,
}

impl Sidebar {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        container.set_width_request(64);
        container.add_css_class("sidebar");

        let btn_servers = Self::create_sidebar_button("network-transmit-receive-symbolic");
        let btn_keys = Self::create_sidebar_button("changes-prevent-symbolic");
        let btn_settings = Self::create_sidebar_button("applications-system-symbolic");
        btn_settings.set_margin_bottom(12);

        let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        spacer.set_vexpand(true);

        container.append(&btn_servers);
        container.append(&btn_keys);
        container.append(&spacer);
        container.append(&btn_settings);

        Self {
            container,
            btn_servers,
            btn_keys,
            btn_settings,
        }
    }

    fn create_sidebar_button(icon_name: &str) -> gtk4::Button {
        let btn = gtk4::Button::from_icon_name(icon_name);
        btn.add_css_class("flat");
        btn.add_css_class("sidebar-button");
        btn.set_halign(gtk4::Align::Center);
        btn.set_valign(gtk4::Align::Start);
        btn.set_width_request(42);
        btn.set_height_request(42);
        btn
    }
}
