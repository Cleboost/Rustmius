use crate::config_observer::AppConfig;
use gtk4::prelude::*;

pub struct Settings {
    pub container: gtk4::Box,
}

impl Settings {
    pub fn new(notebook: &gtk4::Notebook) -> Self {
        let config = crate::config_observer::load_app_config().unwrap_or_else(|e| {
            tracing::error!("Failed to load app config: {}", e);
            AppConfig::default()
        });
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);

        let scrolled = gtk4::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 24);
        content.add_css_class("page");
        content.set_halign(gtk4::Align::Center);
        content.set_width_request(560);

        let header_box = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        header_box.add_css_class("page-header");
        header_box.append(
            &gtk4::Label::builder()
                .label("Settings")
                .halign(gtk4::Align::Start)
                .css_classes(vec!["title-1".to_string()])
                .build(),
        );
        header_box.append(
            &gtk4::Label::builder()
                .label("Configure your global preferences")
                .halign(gtk4::Align::Start)
                .css_classes(vec!["page-subtitle".to_string()])
                .build(),
        );
        content.append(&header_box);

        let font_dialog = gtk4::FontDialog::new();
        let font_button = gtk4::FontDialogButton::builder()
            .dialog(&font_dialog)
            .build();
        font_button.set_font_desc(&gtk4::pango::FontDescription::from_string(
            &config.terminal_font,
        ));

        let scrollback_adj = gtk4::Adjustment::new(
            config.terminal_scrollback as f64,
            100.0,
            100000.0,
            100.0,
            1000.0,
            0.0,
        );
        let scrollback_spinner = gtk4::SpinButton::new(Some(&scrollback_adj), 1.0, 0);

        let theme_labels: Vec<&str> = crate::ui::theme::THEMES.iter().map(|t| t.name).collect();
        let theme_dropdown = gtk4::DropDown::from_strings(&theme_labels);
        let current_theme_idx = crate::ui::theme::THEMES
            .iter()
            .position(|t| t.name == config.terminal_theme)
            .unwrap_or(0) as u32;
        theme_dropdown.set_selected(current_theme_idx);

        let refresh_dropdown = gtk4::DropDown::from_strings(&["1s", "3s", "5s", "10s"]);
        refresh_dropdown.set_selected(config.monitor_refresh_rate);

        let confirm_switch = gtk4::Switch::new();
        confirm_switch.set_active(config.confirm_tab_close);

        let terminal_group = Self::settings_group("Terminal");
        Self::add_row(&terminal_group, "Font", font_button.clone().upcast());
        Self::add_row(
            &terminal_group,
            "Scrollback Lines",
            scrollback_spinner.clone().upcast(),
        );
        Self::add_row(
            &terminal_group,
            "Color Theme",
            theme_dropdown.clone().upcast(),
        );
        content.append(&terminal_group);

        let monitor_group = Self::settings_group("System Monitor");
        Self::add_row(
            &monitor_group,
            "Default Refresh Rate",
            refresh_dropdown.clone().upcast(),
        );
        content.append(&monitor_group);

        let ui_group = Self::settings_group("User Interface");
        Self::add_row(
            &ui_group,
            "Confirm before closing tabs",
            confirm_switch.clone().upcast(),
        );
        content.append(&ui_group);

        let r_drop = refresh_dropdown.clone();
        let f_btn = font_button.clone();
        let s_spin = scrollback_spinner.clone();
        let t_drop = theme_dropdown.clone();
        let c_switch = confirm_switch.clone();

        let save_config = move || {
            let mut new_config = AppConfig::default();
            new_config.monitor_refresh_rate = r_drop.selected();
            new_config.terminal_font = f_btn
                .font_desc()
                .map(|fd| fd.to_string())
                .unwrap_or_else(|| "Monospace 11".to_string());
            new_config.terminal_scrollback = s_spin.value() as u32;
            new_config.terminal_theme = crate::ui::theme::theme_at(t_drop.selected() as usize)
                .name
                .to_string();
            new_config.confirm_tab_close = c_switch.is_active();
            let _ = crate::config_observer::save_app_config(&new_config);
        };

        let save_fn = std::rc::Rc::new(save_config);
        let s1 = save_fn.clone();
        refresh_dropdown.connect_selected_notify(move |_| {
            s1();
        });
        let s2 = save_fn.clone();
        font_button.connect_font_desc_notify(move |_| {
            s2();
        });
        let s3 = save_fn.clone();
        scrollback_spinner.connect_value_changed(move |_| {
            s3();
        });
        let s5 = save_fn.clone();
        confirm_switch.connect_active_notify(move |_| {
            s5();
        });

        let s4 = save_fn.clone();
        let nb = notebook.clone();
        theme_dropdown.connect_selected_notify(move |dd| {
            s4();
            crate::ui::theme::apply_to_open_terminals(
                &nb,
                crate::ui::theme::theme_at(dd.selected() as usize),
            );
        });

        scrolled.set_child(Some(&content));
        container.append(&scrolled);

        Self { container }
    }

    fn settings_group(title: &str) -> gtk4::Box {
        let group = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        group.add_css_class("settings-group");
        group.append(
            &gtk4::Label::builder()
                .label(title)
                .halign(gtk4::Align::Start)
                .css_classes(vec!["settings-group-title".to_string()])
                .build(),
        );
        group
    }

    fn add_row(group: &gtk4::Box, label_text: &str, control: gtk4::Widget) {
        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        row.add_css_class("settings-row");
        let label = gtk4::Label::new(Some(label_text));
        label.set_hexpand(true);
        label.set_halign(gtk4::Align::Start);
        row.append(&label);
        row.append(&control);
        group.append(&row);
    }
}
