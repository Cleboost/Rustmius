pub mod add_server_dialog;
pub mod components;
pub mod docker;
pub mod file_explorer;
pub mod monitor;
pub mod server_list;
pub mod ssh_keys;
pub mod style;
pub mod theme;
pub mod window;

use gtk4::prelude::*;
use gtk4::{gio, glib};

/// Set hand cursor on widgets that act like buttons but aren't gtk::Button.
pub fn set_pointer_cursor(widget: &impl IsA<gtk4::Widget>) {
    widget.set_cursor_from_name(Some("pointer"));
}

pub fn get_docker_icon() -> gtk4::Image {
    let bytes = include_bytes!("../assets/docker.svg");
    let gbytes = glib::Bytes::from_static(bytes);
    let icon = gio::BytesIcon::new(&gbytes);
    let image = gtk4::Image::from_gicon(&icon);
    image.set_pixel_size(16);
    image.add_css_class("docker-icon");
    image
}

pub fn get_container_icon() -> gtk4::Image {
    let bytes = include_bytes!("../assets/container.svg");
    let gbytes = glib::Bytes::from_static(bytes);
    let icon = gio::BytesIcon::new(&gbytes);
    let image = gtk4::Image::from_gicon(&icon);
    image.set_pixel_size(16);
    image.add_css_class("container-icon");
    image
}
