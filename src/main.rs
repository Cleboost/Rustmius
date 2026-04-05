mod config_observer;
mod ui;
mod ssh_engine;

use gtk4::prelude::*;
use crate::ui::window::build_ui;

fn main() {
    let app = gtk4::Application::builder()
        .application_id("org.rustmius.Rustmius")
        .build();

    app.connect_activate(build_ui);
    app.run();
}
