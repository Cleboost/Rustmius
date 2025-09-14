use gtk4::{
    Box, Button, Dialog, Entry, Orientation,
    prelude::{BoxExt, ButtonExt, EditableExt, GtkWindowExt, WidgetExt},
};

pub fn create_rename_dialog(
    title: &str,
    initial_text: &str,
    on_save: Option<std::boxed::Box<dyn Fn(String) + 'static>>,
    parent_window: Option<&gtk4::Window>,
) -> Dialog {
    let dialog = Dialog::builder()
        .title(title)
        .modal(true)
        .resizable(false)
        .decorated(true)
        .build();

    if let Some(parent) = parent_window {
        dialog.set_transient_for(Some(parent));
    }

    let content_box = Box::new(Orientation::Vertical, 12);
    content_box.set_margin_top(20);
    content_box.set_margin_bottom(20);
    content_box.set_margin_start(24);
    content_box.set_margin_end(24);

    let entry = Entry::builder().text(initial_text).build();
    content_box.append(&entry);

    dialog.set_child(Some(&content_box));

    let save_button = Button::builder()
        .label("Save")
        .css_classes(vec!["suggested-action"])
        .build();

    content_box.append(&save_button);

    save_button.connect_clicked({
        let entry = entry.clone();
        let dialog = dialog.clone();
        let on_save_cb = on_save;
        move |_| {
            let text = entry.text().to_string();
            if let Some(cb) = &on_save_cb {
                cb(text);
            }
            dialog.close();
        }
    });

    dialog
}
