use gtk4::prelude::*;
use crate::ui::server_list::ServerList;
use vte4::prelude::*;

pub fn build_ui(app: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Rustmius")
        .default_width(1100)
        .default_height(800)
        .build();

    let root = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    // 1. Sidebar
    let sidebar = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
    sidebar.set_width_request(60);
    sidebar.set_margin_top(12);

    let btn_servers = gtk4::Button::from_icon_name("network-server-symbolic");
    btn_servers.add_css_class("flat");
    
    let btn_keys = gtk4::Button::from_icon_name("key-symbolic");
    btn_keys.add_css_class("flat");

    let spacer = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    spacer.set_vexpand(true);

    let btn_settings = gtk4::Button::from_icon_name("emblem-system-symbolic");
    btn_settings.add_css_class("flat");

    sidebar.append(&btn_servers);
    sidebar.append(&btn_keys);
    sidebar.append(&spacer);
    sidebar.append(&btn_settings);

    let separator = gtk4::Separator::new(gtk4::Orientation::Vertical);

    // 2. Content Area
    let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
    content_box.set_hexpand(true);

    let header = gtk4::HeaderBar::new();
    let add_btn = gtk4::Button::from_icon_name("list-add-symbolic");
    add_btn.add_css_class("suggested-action");
    header.pack_start(&add_btn);
    
    let stack = gtk4::Stack::new();
    stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

    content_box.append(&header);
    content_box.append(&stack);

    root.append(&sidebar);
    root.append(&separator);
    root.append(&content_box);

    window.set_child(Some(&root));
    window.present();
}
