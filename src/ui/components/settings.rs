use gtk4::prelude::*;
use crate::config_observer::AppConfig;

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
        
        let content = gtk4::Box::new(gtk4::Orientation::Vertical, 32);
        content.set_margin_top(48); content.set_margin_bottom(48); content.set_margin_start(48); content.set_margin_end(48);
        content.set_halign(gtk4::Align::Center);
        content.set_width_request(600);

        let header_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        header_box.append(&gtk4::Label::builder().label("Settings").halign(gtk4::Align::Start).css_classes(vec!["title-1".to_string()]).build());
        header_box.append(&gtk4::Label::builder().label("Configure your global preferences").halign(gtk4::Align::Start).css_classes(vec!["dim-label".to_string()]).build());
        content.append(&header_box);

        let terminal_group = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        terminal_group.append(&gtk4::Label::builder().label("Terminal").halign(gtk4::Align::Start).css_classes(vec!["title-4".to_string()]).build());

        let font_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let font_label = gtk4::Label::new(Some("Font")); font_label.set_hexpand(true); font_label.set_halign(gtk4::Align::Start);
        let font_dialog = gtk4::FontDialog::new();
        let font_button = gtk4::FontDialogButton::builder()
            .dialog(&font_dialog)
            .build();
        let initial_font = gtk4::pango::FontDescription::from_string(&config.terminal_font);
        font_button.set_font_desc(&initial_font);
        font_row.append(&font_label); font_row.append(&font_button);
        terminal_group.append(&font_row);

        let scrollback_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let scrollback_label = gtk4::Label::new(Some("Scrollback Lines")); scrollback_label.set_hexpand(true); scrollback_label.set_halign(gtk4::Align::Start);
        let scrollback_adj = gtk4::Adjustment::new(config.terminal_scrollback as f64, 100.0, 100000.0, 100.0, 1000.0, 0.0);
        let scrollback_spinner = gtk4::SpinButton::new(Some(&scrollback_adj), 1.0, 0);
        scrollback_row.append(&scrollback_label); scrollback_row.append(&scrollback_spinner);
        terminal_group.append(&scrollback_row);

        let theme_labels: Vec<&str> = crate::ui::theme::THEMES.iter().map(|t| t.name).collect();
        let theme_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let theme_label = gtk4::Label::new(Some("Color Theme")); theme_label.set_hexpand(true); theme_label.set_halign(gtk4::Align::Start);
        let theme_dropdown = gtk4::DropDown::from_strings(&theme_labels);
        let current_theme_idx = crate::ui::theme::THEMES
            .iter()
            .position(|t| t.name == config.terminal_theme)
            .unwrap_or(0) as u32;
        theme_dropdown.set_selected(current_theme_idx);
        theme_row.append(&theme_label); theme_row.append(&theme_dropdown);
        terminal_group.append(&theme_row);
        content.append(&terminal_group);

        let monitor_group = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        monitor_group.append(&gtk4::Label::builder().label("System Monitor").halign(gtk4::Align::Start).css_classes(vec!["title-4".to_string()]).build());

        let refresh_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let refresh_label = gtk4::Label::new(Some("Default Refresh Rate")); refresh_label.set_hexpand(true); refresh_label.set_halign(gtk4::Align::Start);
        let refresh_dropdown = gtk4::DropDown::from_strings(&["1s", "3s", "5s", "10s"]);
        refresh_dropdown.set_selected(config.monitor_refresh_rate);
        refresh_row.append(&refresh_label); refresh_row.append(&refresh_dropdown);
        monitor_group.append(&refresh_row);
        content.append(&monitor_group);

        let ui_group = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        ui_group.append(&gtk4::Label::builder().label("User Interface").halign(gtk4::Align::Start).css_classes(vec!["title-4".to_string()]).build());

        let confirm_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let confirm_label = gtk4::Label::new(Some("Confirm before closing tabs")); confirm_label.set_hexpand(true); confirm_label.set_halign(gtk4::Align::Start);
        let confirm_switch = gtk4::Switch::new();
        confirm_switch.set_active(config.confirm_tab_close);
        confirm_row.append(&confirm_label); confirm_row.append(&confirm_switch);
        ui_group.append(&confirm_row);
        content.append(&ui_group);

        let r_drop = refresh_dropdown.clone();
        let f_btn = font_button.clone();
        let s_spin = scrollback_spinner.clone();
        let t_drop = theme_dropdown.clone();
        let c_switch = confirm_switch.clone();

        let save_config = move || {
            let mut new_config = AppConfig::default();
            new_config.monitor_refresh_rate = r_drop.selected();
            new_config.terminal_font = f_btn.font_desc()
                .map(|fd| fd.to_string())
                .unwrap_or_else(|| "Monospace 11".to_string());
            new_config.terminal_scrollback = s_spin.value() as u32;
            new_config.terminal_theme = crate::ui::theme::theme_at(t_drop.selected() as usize).name.to_string();
            new_config.confirm_tab_close = c_switch.is_active();
            let _ = crate::config_observer::save_app_config(&new_config);
        };

        let save_fn = std::rc::Rc::new(save_config);
        let s1 = save_fn.clone(); refresh_dropdown.connect_selected_notify(move |_| { s1(); });
        let s2 = save_fn.clone(); font_button.connect_font_desc_notify(move |_| { s2(); });
        let s3 = save_fn.clone(); scrollback_spinner.connect_value_changed(move |_| { s3(); });
        let s5 = save_fn.clone(); confirm_switch.connect_active_notify(move |_| { s5(); });

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
}
