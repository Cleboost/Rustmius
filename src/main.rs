mod config_observer;
mod engines;
mod ui;

use crate::ui::window::build_ui;
use gtk4::prelude::*;
use tracing::info;

#[tokio::main]
async fn main() {
    let is_askpass = std::env::var("RUSTMIUS_ASKPASS_ALIAS").is_ok();

    if !is_askpass {
        tracing_subscriber::fmt()
            .with_writer(std::io::stderr)
            .with_env_filter(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive(tracing::Level::INFO.into())
                    .add_directive("rustmius=debug".parse().unwrap()),
            )
            .init();
        info!("Starting Rustmius v{}", env!("CARGO_PKG_VERSION"));
    }

    if let Ok(alias) = std::env::var("RUSTMIUS_ASKPASS_ALIAS") {
        if let Some(pass_str) = crate::config_observer::get_keyring_password(&alias).await {
            // Wipe the plaintext password from memory once it has been written out.
            let pass_str = zeroize::Zeroizing::new(pass_str);
            use std::io::Write;
            let mut stdout = std::io::stdout().lock();
            let _ = stdout.write_all(pass_str.as_bytes());
            let _ = stdout.flush();
            std::process::exit(0);
        }
        std::process::exit(1);
    }

    let app = gtk4::Application::builder()
        .application_id("org.rustmius.Rustmius")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(build_ui);
    app.run_with_args::<&str>(&[]);
}
