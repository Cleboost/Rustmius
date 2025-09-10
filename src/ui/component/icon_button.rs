use gtk4::{Button, prelude::ButtonExt};
use libadwaita::ButtonContent;

pub fn create_icon_button(
    label: &str,
    icon_name: &str,
    width: i32,
    height: i32,
    callback: impl Fn() + 'static,
) -> Button {
    let content = ButtonContent::builder()
        .label(label)
        .icon_name(icon_name)
        .build();

    let button = Button::builder()
        .child(&content)
        .width_request(width)
        .height_request(height)
        .build();

    button.connect_clicked(move |_| {
        callback();
    });

    button
}
