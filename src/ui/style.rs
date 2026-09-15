use gtk4::CssProvider;
use gtk4::gdk;
use gtk4::gio;
use gtk4::gio::prelude::*;

const PORTAL_BUS_NAME: &str = "org.freedesktop.portal.Desktop";
const PORTAL_OBJECT_PATH: &str = "/org/freedesktop/portal/desktop";
const PORTAL_SETTINGS_IFACE: &str = "org.freedesktop.portal.Settings";
const APPEARANCE_NAMESPACE: &str = "org.freedesktop.appearance";
const COLOR_SCHEME_KEY: &str = "color-scheme";

/// Peels away GVariant `v` (variant) wrapper layers until a non-variant value
/// is reached. Some portal backends (notably xdg-desktop-portal-gnome) wrap
/// the `Read`/`SettingChanged` payload in an extra variant layer on top of
/// the `v` already mandated by the portal spec, so a single unwrap isn't
/// always enough.
fn unwrap_variant(mut value: gtk4::glib::Variant) -> gtk4::glib::Variant {
    while let Some(inner) = value.as_variant() {
        value = inner;
    }
    value
}

/// Maps the portal's `color-scheme` value (0 = no preference, 1 = prefer
/// dark, 2 = prefer light) to a `prefer-dark-theme` bool, if it expresses one.
fn color_scheme_prefers_dark(value: u32) -> Option<bool> {
    match value {
        1 => Some(true),
        2 => Some(false),
        _ => None,
    }
}

fn apply_color_scheme(value: u32) {
    tracing::debug!(value, "portal color-scheme received");
    match color_scheme_prefers_dark(value) {
        Some(prefer_dark) => match gtk4::Settings::default() {
            Some(settings) => {
                settings.set_gtk_application_prefer_dark_theme(prefer_dark);
                tracing::debug!(
                    prefer_dark,
                    now = settings.is_gtk_application_prefer_dark_theme(),
                    "applied gtk-application-prefer-dark-theme"
                );
            }
            None => tracing::warn!("no default GtkSettings available (no display?)"),
        },
        None => tracing::debug!(value, "color-scheme value expresses no preference"),
    }
}

/// Syncs the app's light/dark mode with the desktop's system-wide preference
/// via the XDG Desktop Portal, since plain GTK4 (without libadwaita) does not
/// do this on its own. Keeps following live changes for the rest of the run.
pub fn init_theme_sync() {
    let proxy = match gio::DBusProxy::for_bus_sync(
        gio::BusType::Session,
        gio::DBusProxyFlags::NONE,
        None,
        PORTAL_BUS_NAME,
        PORTAL_OBJECT_PATH,
        PORTAL_SETTINGS_IFACE,
        gio::Cancellable::NONE,
    ) {
        Ok(proxy) => proxy,
        Err(err) => {
            tracing::warn!(%err, "could not connect to xdg-desktop-portal Settings interface");
            return;
        }
    };

    match proxy.call_sync(
        "Read",
        Some(&(APPEARANCE_NAMESPACE, COLOR_SCHEME_KEY).to_variant()),
        gio::DBusCallFlags::NONE,
        1000,
        gio::Cancellable::NONE,
    ) {
        Ok(reply) => match unwrap_variant(reply.child_value(0)).get::<u32>() {
            Some(value) => apply_color_scheme(value),
            None => tracing::warn!(reply = %reply, "unexpected reply shape from portal Read"),
        },
        Err(err) => tracing::warn!(%err, "portal Settings.Read call failed"),
    }

    proxy.connect_g_signal(move |_proxy, _sender, signal, params| {
        if signal != "SettingChanged" {
            return;
        }
        if let Some((namespace, key, value)) = params.get::<(String, String, gtk4::glib::Variant)>()
            && namespace == APPEARANCE_NAMESPACE
            && key == COLOR_SCHEME_KEY
            && let Some(value) = unwrap_variant(value).get::<u32>()
        {
            apply_color_scheme(value);
        }
    });
}

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
