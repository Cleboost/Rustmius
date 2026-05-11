use gtk4::gdk;
use gtk4::CssProvider;

pub fn init_style() {
    let provider = CssProvider::new();
    provider.load_from_string("
        .docker-card, .action-card, .card {
            background-color: alpha(@theme_fg_color, 0.05);
            border: 1px solid alpha(@theme_fg_color, 0.1);
            border-radius: 12px;
            padding: 16px;
            transition: all 200ms;
        }
        .docker-card:hover, .action-card:hover, .card:hover {
            background-color: alpha(@theme_fg_color, 0.08);
            border-color: alpha(@theme_fg_color, 0.2);
        }
        .stat-value { font-size: 2.2em; font-weight: 800; }
        .sftp-list row { padding: 6px 12px; border-radius: 6px; margin: 2px 0; }
        .sftp-list row:selected { background-color: @theme_selected_bg_color; color: @theme_selected_fg_color; }
        .sftp-list row:hover:not(:selected) { background-color: alpha(@theme_fg_color, 0.04); }
        .sftp-status { padding: 6px 12px; background-color: alpha(@theme_fg_color, 0.03); border-top: 1px solid alpha(@theme_fg_color, 0.08); }
        .sftp-path { font-family: monospace; font-size: 0.92em; padding: 4px; }
        .file-size { opacity: 0.55; font-size: 0.85em; font-family: monospace; }
        
        .dim-label { opacity: 0.65; }
        .bold { font-weight: bold; }
        .heading { font-weight: 800; font-size: 1.25em; }
        .caption { font-size: 0.88em; }
        .title-1 { font-size: 2.6em; font-weight: 800; margin-bottom: 8px; }
        .title-2 { font-size: 2.1em; font-weight: 800; margin-bottom: 6px; }
        .title-3 { font-size: 1.6em; font-weight: 700; }
        .title-4 { font-size: 1.15em; font-weight: 700; }

        .boxed-list {
            border: 1px solid alpha(@theme_fg_color, 0.1);
            border-radius: 10px;
            background-color: alpha(@theme_bg_color, 0.5);
        }

        /* Sidebar styling */
        .sidebar {
            background-color: alpha(@theme_fg_color, 0.02);
            padding: 8px;
        }
        .sidebar-button {
            border-radius: 8px;
            padding: 10px;
            transition: all 150ms;
        }
    ");
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
