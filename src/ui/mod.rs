pub mod style;
pub mod window;
pub mod server_list;
pub mod components;
pub mod add_server_dialog;
pub mod file_explorer;
pub mod ssh_keys;
pub mod monitor;
pub mod docker;

use gtk4::{gio, glib};

pub fn get_docker_icon() -> gtk4::Image {
    let bytes = include_bytes!("../assets/docker.svg");
    let gbytes = glib::Bytes::from_static(bytes);
    let icon = gio::BytesIcon::new(&gbytes);
    let image = gtk4::Image::from_gicon(&icon);
    image.set_pixel_size(16);
    image
}

pub fn get_container_icon() -> gtk4::Image {
    let bytes = include_bytes!("../assets/container.svg");
    let gbytes = glib::Bytes::from_static(bytes);
    let icon = gio::BytesIcon::new(&gbytes);
    let image = gtk4::Image::from_gicon(&icon);
    image.set_pixel_size(16);
    image
}