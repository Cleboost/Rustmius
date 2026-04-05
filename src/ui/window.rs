use gtk4::prelude::*;
use vte4::prelude::*;

pub fn build_ui(app: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Rustmius")
        .default_width(1000)
        .default_height(700)
        .build();

    let terminal = vte4::Terminal::new();
    
    // Some basic terminal settings
    terminal.set_cursor_blink_mode(vte4::CursorBlinkMode::On);
    terminal.set_scrollback_lines(10000);

    window.set_child(Some(&terminal));
    window.present();
}
