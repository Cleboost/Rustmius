use gtk4::prelude::*;

#[derive(Clone)]
pub struct Sidebar {
    pub container: gtk4::Box,
    pub btn_servers: gtk4::Button,
    pub btn_keys: gtk4::Button,
    pub btn_settings: gtk4::Button,
}

impl Sidebar {
    pub fn new() -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        container.set_width_request(60);
        container.add_css_class("sidebar");

        let btn_servers = Self::create_sidebar_button("network-transmit-receive-symbolic");
        let btn_keys = Self::create_sidebar_button("changes-prevent-symbolic");
        let btn_settings = Self::create_sidebar_button("applications-system-symbolic");

        btn_servers.add_css_class("active");

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

    pub fn set_active(&self, page: &str) {
        self.btn_servers.remove_css_class("active");
        self.btn_keys.remove_css_class("active");
        self.btn_settings.remove_css_class("active");
        match page {
            "sessions" => self.btn_servers.add_css_class("active"),
            "ssh_keys" => self.btn_keys.add_css_class("active"),
            "settings" => self.btn_settings.add_css_class("active"),
            _ => {}
        }
    }

    fn create_sidebar_button(icon_name: &str) -> gtk4::Button {
        let btn = gtk4::Button::from_icon_name(icon_name);
        btn.add_css_class("flat");
        btn.add_css_class("sidebar-button");
        btn.set_halign(gtk4::Align::Center);
        btn.set_valign(gtk4::Align::Start);
        btn.set_width_request(40);
        btn.set_height_request(40);
        crate::ui::set_pointer_cursor(&btn);
        btn
    }
}
