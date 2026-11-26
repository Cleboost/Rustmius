mod app;
mod core;
mod i18n;
mod storage;
mod ui;

use app::RustmiusApp;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([900.0, 560.0])
            .with_decorations(false)
            .with_title("Rustmius"),
        ..Default::default()
    };

    eframe::run_native(
        "Rustmius",
        native_options,
        Box::new(|cc| Ok(Box::new(RustmiusApp::new(cc)))),
    )
}
