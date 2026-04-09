mod config_observer;
mod ui;
mod ssh_engine;
mod sftp_engine;

use gtk4::prelude::*;
use crate::ui::window::build_ui;
use std::fs::OpenOptions;
use std::io::Write;

fn log_debug(msg: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("/tmp/rustmius_debug.log") {
        let _ = writeln!(file, "[{}] {}", chrono::Local::now(), msg);
    }
}

#[tokio::main]
async fn main() {
    let _args: Vec<String> = std::env::args().collect();
    
    if let Ok(alias) = std::env::var("RUSTMIUS_ASKPASS_ALIAS") {
        log_debug(&format!("AskPass triggered for alias: {}", alias));
        if let Ok(keyring) = oo7::Keyring::new().await {
            let mut attributes = std::collections::HashMap::new();
            let normalized_alias = alias.to_lowercase();
            attributes.insert("rustmius-server-alias", normalized_alias.as_str());
            if let Ok(items) = keyring.search_items(&attributes).await {
                log_debug(&format!("Found {} items in keyring", items.len()));
                if let Some(item) = items.first() {
                    if let Ok(password) = item.secret().await {
                        if let Ok(pass_str) = std::str::from_utf8(&password) {
                            log_debug("Password retrieved successfully, sending to SSH");
                            
                            print!("{}", pass_str);
                            std::process::exit(0);
                        }
                    }
                }
            }
        }
        log_debug("Failed to retrieve password from keyring");
        std::process::exit(1);
    }

    let app = gtk4::Application::builder()
        .application_id("org.rustmius.Rustmius")
        .flags(gtk4::gio::ApplicationFlags::NON_UNIQUE)
        .build();

    app.connect_activate(build_ui);
    app.run_with_args::<&str>(&[]);
}
