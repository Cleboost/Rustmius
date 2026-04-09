use gtk4::prelude::*;
fn main() {
    let formats = gtk4::gdk::ContentFormats::builder().add_type(gtk4::gdk::FileList::static_type()).build();
    let _target = gtk4::DropTarget::builder().formats(&formats).build();
}
