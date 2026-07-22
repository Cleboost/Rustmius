use gtk4::CssProvider;
use gtk4::gdk;

pub fn init_style() {
    let provider = CssProvider::new();
    provider.load_from_string(
        "
        /* ── Cards ─────────────────────────────────────────── */
        .card, .docker-card, .action-card, .settings-group, .monitor-card {
            background-color: alpha(@theme_fg_color, 0.04);
            border: 1px solid alpha(@theme_fg_color, 0.07);
            border-radius: 14px;
            transition: background-color 180ms ease, border-color 180ms ease, box-shadow 180ms ease;
        }
        .card, .docker-card, .action-card {
            padding: 20px;
        }
        .card:hover, .docker-card:hover, .action-card:hover {
            background-color: alpha(@theme_fg_color, 0.06);
            border-color: alpha(@theme_fg_color, 0.12);
            box-shadow: 0 2px 12px alpha(@theme_fg_color, 0.06);
        }
        .card { cursor: pointer; }

        /* ── Typography ────────────────────────────────────── */
        .title-1 {
            font-size: 1.75em;
            font-weight: 700;
            letter-spacing: -0.02em;
        }
        .title-2 {
            font-size: 1.35em;
            font-weight: 700;
            letter-spacing: -0.01em;
        }
        .title-3 {
            font-size: 1.1em;
            font-weight: 600;
        }
        .title-4 {
            font-size: 0.95em;
            font-weight: 600;
        }
        .heading {
            font-weight: 600;
            font-size: 1.05em;
            letter-spacing: -0.01em;
        }
        .caption {
            font-size: 0.82em;
        }
        .dim-label {
            opacity: 0.55;
        }
        .bold { font-weight: 600; }
        .stat-value {
            font-size: 2em;
            font-weight: 700;
            letter-spacing: -0.03em;
        }
        .page-subtitle {
            font-size: 0.9em;
            opacity: 0.55;
        }

        /* ── Page layout ───────────────────────────────────── */
        .page {
            margin: 32px 40px;
        }
        .page-header {
            margin-bottom: 28px;
        }
        .page-header-actions button {
            min-width: 36px;
            min-height: 36px;
        }

        /* ── Settings ──────────────────────────────────────── */
        .settings-group {
            padding: 20px 24px;
        }
        .settings-group-title {
            font-weight: 600;
            font-size: 0.85em;
            opacity: 0.55;
            text-transform: uppercase;
            letter-spacing: 0.04em;
            margin-bottom: 4px;
        }
        .settings-row {
            padding: 10px 0;
            border-bottom: 1px solid alpha(@theme_fg_color, 0.05);
        }
        .settings-row:last-child {
            border-bottom: none;
            padding-bottom: 0;
        }
        .settings-row:first-child {
            padding-top: 0;
        }

        /* ── Lists ─────────────────────────────────────────── */
        .boxed-list {
            border: 1px solid alpha(@theme_fg_color, 0.07);
            border-radius: 12px;
            background-color: alpha(@theme_fg_color, 0.02);
        }
        .boxed-list row, .list-row {
            padding: 0;
            border-bottom: 1px solid alpha(@theme_fg_color, 0.04);
        }
        .boxed-list row:last-child, .list-row:last-child {
            border-bottom: none;
        }
        .list-row-content {
            padding: 14px 18px;
        }
        .boxed-list row:selected {
            background-color: alpha(@theme_selected_bg_color, 0.85);
            color: @theme_selected_fg_color;
        }
        .boxed-list row:hover:not(:selected) {
            background-color: alpha(@theme_fg_color, 0.03);
        }

        /* ── SFTP ──────────────────────────────────────────── */
        .sftp-list row {
            padding: 0;
            border-radius: 0;
            margin: 0;
        }
        .sftp-list row:selected {
            background-color: alpha(@theme_selected_bg_color, 0.85);
            color: @theme_selected_fg_color;
        }
        .sftp-list row:hover:not(:selected) {
            background-color: alpha(@theme_fg_color, 0.03);
        }
        .sftp-status {
            padding: 8px 16px;
            background-color: alpha(@theme_fg_color, 0.02);
            border-top: 1px solid alpha(@theme_fg_color, 0.06);
            font-size: 0.82em;
        }
        .sftp-path {
            font-family: monospace;
            font-size: 0.88em;
            padding: 6px 12px;
            border-radius: 8px;
            background-color: alpha(@theme_fg_color, 0.04);
            border: 1px solid alpha(@theme_fg_color, 0.06);
        }
        .path-bar {
            padding: 8px 12px;
            border-bottom: 1px solid alpha(@theme_fg_color, 0.06);
            background-color: alpha(@theme_fg_color, 0.015);
        }
        .path-bar button {
            min-width: 32px;
            min-height: 32px;
            padding: 4px;
            border-radius: 8px;
        }
        .file-size {
            opacity: 0.45;
            font-size: 0.82em;
            font-family: monospace;
        }

        /* ── Sidebar ───────────────────────────────────────── */
        .sidebar {
            background-color: alpha(@theme_fg_color, 0.015);
            padding: 12px 10px;
            border-right: 1px solid alpha(@theme_fg_color, 0.06);
        }
        .sidebar-button {
            border-radius: 10px;
            padding: 0;
            min-width: 40px;
            min-height: 40px;
            transition: background-color 150ms ease, color 150ms ease;
        }
        .sidebar-button:hover {
            background-color: alpha(@theme_fg_color, 0.06);
        }
        .sidebar-button.active {
            background-color: alpha(@theme_selected_bg_color, 0.15);
            color: @theme_selected_bg_color;
        }
        .sidebar-button.active:hover {
            background-color: alpha(@theme_selected_bg_color, 0.22);
        }

        /* ── Session toolbar ───────────────────────────────── */
        .session-toolbar {
            padding: 6px 12px;
            border-bottom: 1px solid alpha(@theme_fg_color, 0.06);
            background-color: alpha(@theme_fg_color, 0.015);
        }
        .session-toolbar button {
            min-width: 34px;
            min-height: 34px;
            padding: 4px;
            border-radius: 8px;
        }
        .session-toolbar .docker-icon,
        .session-toolbar .container-icon {
            color: alpha(@theme_fg_color, 0.8);
        }
        .session-toolbar button:hover .docker-icon,
        .session-toolbar button:hover .container-icon {
            color: @theme_fg_color;
        }
        .docker-icon,
        .container-icon {
            color: @theme_fg_color;
        }

        /* ── Header ────────────────────────────────────────── */
        .main-headerbar {
            padding-left: 0;
            padding-right: 8px;
        }
        .main-headerbar > box.start {
            margin: 0;
            padding: 0;
        }
        headerbar box.start {
            margin-left: 0;
            padding-left: 0;
        }
        .header-add-btn {
            min-width: 36px;
            min-height: 36px;
            border-radius: 10px;
        }

        /* ── Cursors ───────────────────────────────────────── */
        button,
        .sidebar-button,
        .sidebar button,
        .card,
        .action-card,
        .clickable-row,
        .tab-label,
        .card label,
        .session-notebook > header > tabs > tab,
        .session-notebook > header > tabs > tab button,
        .session-notebook > header > tabs > arrow,
        .tab-close-btn,
        .session-toolbar button,
        .path-bar button,
        .page-header-actions button,
        .server-card-actions button,
        switch {
            cursor: pointer;
        }
        entry,
        spinbutton,
        textview,
        vte-terminal {
            cursor: text;
        }

        /* ── Notebook tabs ─────────────────────────────────── */
        .session-notebook {
            background-color: transparent;
        }
        .session-notebook > header {
            padding: 5px 8px 0;
            min-height: 0;
            border-bottom: 1px solid alpha(@theme_fg_color, 0.06);
            background-color: alpha(@theme_fg_color, 0.012);
            box-shadow: none;
        }
        .session-notebook > header > tabs {
            margin: 0;
            padding: 0;
            border: none;
            box-shadow: none;
            -gtk-tab-overlap: 0;
        }
        .session-notebook > header > tabs > tab {
            padding: 0;
            margin: 0 2px;
            min-width: 72px;
            min-height: 0;
            border: none;
            border-radius: 7px 7px 0 0;
            background: transparent;
            box-shadow: none;
            outline: none;
            transition: background-color 120ms ease;
        }
        .session-notebook > header > tabs > tab:checked {
            background-color: alpha(@theme_fg_color, 0.07);
        }
        .session-notebook > header > tabs > tab:hover:not(:checked) {
            background-color: alpha(@theme_fg_color, 0.035);
        }
        .session-notebook > header > tabs > arrow {
            min-width: 18px;
            min-height: 18px;
            padding: 2px;
            margin: 0 1px;
            border-radius: 5px;
        }
        .session-notebook > header > tabs > arrow:hover {
            background-color: alpha(@theme_fg_color, 0.06);
        }

        .tab-label {
            padding: 5px 10px 5px 8px;
        }
        .tab-label .tab-text {
            font-size: 0.82em;
            font-weight: 500;
            opacity: 0.8;
        }
        .session-notebook tab:checked .tab-label .tab-text {
            opacity: 1;
            font-weight: 600;
        }
        .tab-label .tab-icon {
            opacity: 0.65;
        }
        .session-notebook tab:checked .tab-label .tab-icon {
            opacity: 0.9;
        }
        .tab-close-btn {
            min-width: 20px;
            min-height: 20px;
            padding: 0;
            margin: 0 0 0 6px;
            border-radius: 5px;
            color: alpha(@theme_fg_color, 0.7);
            background-color: alpha(@theme_fg_color, 0.06);
            transition: color 120ms ease, background-color 120ms ease;
        }
        .tab-close-btn .tab-close-icon {
            opacity: 1;
        }
        .session-notebook tab:checked .tab-label .tab-close-btn {
            color: alpha(@theme_fg_color, 0.85);
            background-color: alpha(@theme_fg_color, 0.08);
        }
        .tab-label:hover .tab-close-btn {
            color: alpha(@theme_fg_color, 0.95);
            background-color: alpha(@theme_fg_color, 0.1);
        }
        .tab-close-btn:hover {
            color: @theme_fg_color;
            background-color: alpha(@theme_fg_color, 0.14);
        }
        .tab-close-btn:active {
            background-color: alpha(@theme_fg_color, 0.2);
        }

        /* ── Monitor ───────────────────────────────────────── */
        .monitor-card {
            padding: 20px;
        }
        .monitor-card .frame-title {
            font-size: 0.82em;
            font-weight: 600;
            opacity: 0.55;
            text-transform: uppercase;
            letter-spacing: 0.03em;
        }
        .info-grid {
            padding: 4px 0;
        }

        /* ── Server card ───────────────────────────────────── */
        .server-card-icon {
            opacity: 0.7;
        }
        .server-card-actions button {
            min-width: 30px;
            min-height: 30px;
            padding: 2px;
            border-radius: 8px;
            opacity: 0.6;
        }
        .server-card-actions button:hover {
            opacity: 1;
        }

        /* ── Dialog ────────────────────────────────────────── */
        dialog .dialog-content {
            padding: 20px 24px;
        }
        dialog label.field-label {
            font-size: 0.85em;
            font-weight: 500;
            opacity: 0.7;
            margin-bottom: 2px;
        }

        /* ── Separator ─────────────────────────────────────── */
        .sidebar-separator {
            opacity: 0.3;
        }
    ",
    );
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}
