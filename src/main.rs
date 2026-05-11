mod config_observer;
mod ui;
mod ssh_engine;
mod sftp_engine;

use gtk4::prelude::*;
use crate::ui::window::build_ui;
use tracing::{debug, info, error};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env()
            .add_directive(tracing::Level::INFO.into())
            .add_directive("rustmius=debug".parse().unwrap()))
        .init();

    info!("Starting Rustmius v{}", env!("CARGO_PKG_VERSION"));

    let _args: Vec<String> = std::env::args().collect();
    if let Ok(alias) = std::env::var("RUSTMIUS_ASKPASS_ALIAS") {
        debug!("AskPass triggered for alias: {}", alias);
        if let Some(pass_str) = tokio::runtime::Handle::current().block_on(crate::config_observer::get_keyring_password(&alias)) {
            debug!("Password retrieved successfully, sending to SSH");
            print!("{}", pass_str);
            std::process::exit(0);
        }
        error!("Failed to retrieve password from keyring for alias: {}", alias);
        std::process::exit(1);
    }

    let app = gtk4::Application::builder()
        .application_id("org.rustmius.Rustmius")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(build_ui);
    app.run_with_args::<&str>(&[]);
}