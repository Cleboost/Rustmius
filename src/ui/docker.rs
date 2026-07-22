use crate::config_observer::SshHost;
use crate::engines::docker::{get_docker_stats, list_docker_items, perform_docker_action};
use gtk4::glib;
use gtk4::prelude::*;

pub struct DockerManager {
    pub container: gtk4::Box,
    host: SshHost,
    password: Option<String>,
    stack: gtk4::Stack,
}

impl DockerManager {
    pub fn new(host: SshHost, password: Option<String>) -> Self {
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let stack = gtk4::Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);

        let manager = Self {
            container,
            host,
            password,
            stack,
        };

        manager.init_ui();
        manager
    }

    fn init_ui(&self) {
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();

        let loading_box = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        loading_box.set_valign(gtk4::Align::Center);
        loading_box.set_halign(gtk4::Align::Center);
        let spinner = gtk4::Spinner::new();
        spinner.set_spinning(true);
        spinner.set_size_request(48, 48);
        let loading_label = gtk4::Label::builder()
            .label("Connecting to Docker...")
            .css_classes(vec!["dim-label".to_string()])
            .build();
        loading_box.append(&spinner);
        loading_box.append(&loading_label);
        self.stack.add_named(&loading_box, Some("loading"));
        self.stack.set_visible_child_name("loading");

        let dashboard = self.build_dashboard();
        self.stack.add_named(&dashboard, Some("dashboard"));

        let containers_view = self.build_list_view("Containers");
        self.stack.add_named(&containers_view, Some("containers"));

        let images_view = self.build_list_view("Images");
        self.stack.add_named(&images_view, Some("images"));
        scrolled.set_child(Some(&self.stack));
        self.container.append(&scrolled);

        self.stack.connect_visible_child_notify(move |s| {
            if let Some(name) = s.visible_child_name() {
                if name == "containers" || name == "images" {
                    if let Some(view) = s.visible_child() {
                        if let Some(box_) = view.downcast_ref::<gtk4::Box>() {
                            if let Some(header) = box_.first_child() {
                                let mut next = header.first_child();
                                while let Some(child) = next {
                                    if let Some(btn) = child.downcast_ref::<gtk4::Button>() {
                                        if btn.icon_name()
                                            == Some(glib::GString::from("view-refresh-symbolic"))
                                        {
                                            btn.emit_clicked();
                                            break;
                                        }
                                    }
                                    next = child.next_sibling();
                                }
                            }
                        }
                    }
                }
            }
        });

        // Start on dashboard and refresh
        self.stack.set_visible_child_name("dashboard");
        self.refresh_dashboard();
    }

    fn build_dashboard(&self) -> gtk4::Box {
        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 32);
        box_.set_margin_top(32);
        box_.set_margin_bottom(32);
        box_.set_margin_start(48);
        box_.set_margin_end(48);

        let header_box = gtk4::Box::new(gtk4::Orientation::Vertical, 4);
        let title = gtk4::Label::builder()
            .label("Docker Management")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["title-1".to_string()])
            .build();
        let subtitle = gtk4::Label::builder()
            .label(&format!("Server: {}", self.host.alias))
            .halign(gtk4::Align::Start)
            .css_classes(vec!["dim-label".to_string()])
            .build();
        header_box.append(&title);
        header_box.append(&subtitle);
        box_.append(&header_box);

        let stats_flow = gtk4::FlowBox::new();
        stats_flow.set_selection_mode(gtk4::SelectionMode::None);
        stats_flow.set_column_spacing(18);
        stats_flow.set_row_spacing(18);
        stats_flow.set_max_children_per_line(4);

        stats_flow.append(&self.build_stat_card("Containers", "0", Some("container")));
        stats_flow.append(&self.build_stat_card("Images", "0", Some("view-grid-symbolic")));
        stats_flow.append(&self.build_stat_card(
            "Running",
            "0",
            Some("media-playback-start-symbolic"),
        ));
        stats_flow.append(&self.build_stat_card(
            "Paused",
            "0",
            Some("media-playback-pause-symbolic"),
        ));
        box_.append(&stats_flow);

        let actions_section = gtk4::Box::new(gtk4::Orientation::Vertical, 16);
        let actions_title = gtk4::Label::builder()
            .label("Quick Actions")
            .halign(gtk4::Align::Start)
            .css_classes(vec!["title-3".to_string()])
            .build();
        actions_section.append(&actions_title);

        let actions_grid = gtk4::Box::new(gtk4::Orientation::Horizontal, 18);

        let btn_containers = self.build_action_tile(
            "Manage Containers",
            "View and control your containers",
            Some("container"),
        );
        let btn_images = self.build_action_tile(
            "Manage Images",
            "Browse and prune images",
            Some("view-grid-symbolic"),
        );

        let s_c = self.stack.clone();
        let g_c = gtk4::GestureClick::new();
        g_c.connect_released(move |_, _, _, _| {
            s_c.set_visible_child_name("containers");
        });
        btn_containers.add_controller(g_c);

        let s_i = self.stack.clone();
        let g_i = gtk4::GestureClick::new();
        g_i.connect_released(move |_, _, _, _| {
            s_i.set_visible_child_name("images");
        });
        btn_images.add_controller(g_i);

        actions_grid.append(&btn_containers);
        actions_grid.append(&btn_images);
        actions_section.append(&actions_grid);
        box_.append(&actions_section);

        box_
    }

    fn build_action_tile(&self, title: &str, desc: &str, icon_name: Option<&str>) -> gtk4::Box {
        let tile = gtk4::Box::new(gtk4::Orientation::Horizontal, 16);
        tile.add_css_class("action-card");
        tile.set_hexpand(true);
        tile.set_cursor_from_name(Some("pointer"));

        let icon_img = match icon_name {
            Some("container") => {
                let img = crate::ui::get_container_icon();
                img.set_pixel_size(32);
                img
            }
            Some(name) => {
                let img = gtk4::Image::from_icon_name(name);
                img.set_pixel_size(32);
                img
            }
            None => {
                let img = crate::ui::get_docker_icon();
                img.set_pixel_size(32);
                img
            }
        };

        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        let label_title = gtk4::Label::builder()
            .label(title)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["title-4".to_string()])
            .build();
        let label_desc = gtk4::Label::builder()
            .label(desc)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();

        text_box.append(&label_title);
        text_box.append(&label_desc);

        tile.append(&icon_img);
        tile.append(&text_box);
        tile
    }

    fn build_list_view(&self, title: &str) -> gtk4::Box {
        let box_ = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        box_.set_margin_top(24);
        box_.set_margin_bottom(24);
        box_.set_margin_start(24);
        box_.set_margin_end(24);

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let back_btn = gtk4::Button::from_icon_name("go-previous-symbolic");
        back_btn.add_css_class("flat");
        let s_back = self.stack.clone();
        back_btn.connect_clicked(move |_| {
            s_back.set_visible_child_name("dashboard");
        });

        let title_label = gtk4::Label::builder()
            .label(title)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["title-2".to_string()])
            .build();

        let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.add_css_class("flat");

        header.append(&back_btn);
        header.append(&title_label);
        let spacer = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        spacer.set_hexpand(true);
        header.append(&spacer);
        header.append(&refresh_btn);
        box_.append(&header);

        let scrolled = gtk4::ScrolledWindow::builder().vexpand(true).build();
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::None);
        list_box.add_css_class("boxed-list");
        scrolled.set_child(Some(&list_box));
        box_.append(&scrolled);

        let host = self.host.clone();
        let password = self.password.clone();
        let lb_clone = list_box.clone();
        let title_owned = title.to_string();

        refresh_btn.connect_clicked(move |rb| {
            let h = host.clone();
            let p = password.clone();
            let lb = lb_clone.clone();
            let t = title_owned.clone();
            let rb_inner = rb.clone();

            glib::MainContext::default().spawn_local(async move {
                let h_for_fetch = h.clone();
                let p_for_fetch = p.clone();
                let is_containers = t == "Containers";
                let result =
                    list_docker_items(&h_for_fetch, p_for_fetch.as_deref(), is_containers).await;

                if let Ok(output) = result {
                    while let Some(child) = lb.first_child() {
                        lb.remove(&child);
                    }
                    for line in output.lines() {
                        let parts: Vec<&str> = line.split('\t').collect();
                        let row = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
                        row.set_margin_top(8);
                        row.set_margin_bottom(8);
                        row.set_margin_start(12);
                        row.set_margin_end(12);

                        let text_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
                        let item_name = parts.first().copied().unwrap_or(line).to_string();
                        let name_label = gtk4::Label::builder()
                            .label(&item_name)
                            .halign(gtk4::Align::Start)
                            .css_classes(vec!["bold".to_string()])
                            .build();
                        text_box.append(&name_label);

                        if parts.len() > 1 {
                            let detail = gtk4::Label::builder()
                                .label(&parts[1..].join(" • "))
                                .halign(gtk4::Align::Start)
                                .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
                                .build();
                            text_box.append(&detail);
                        }

                        row.append(&text_box);

                        let spacer_row = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
                        spacer_row.set_hexpand(true);
                        row.append(&spacer_row);

                        let actions = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
                        let is_container = t == "Containers";

                        if is_container {
                            let status = parts.get(1).copied().unwrap_or("");
                            let is_running = status.starts_with("Up");
                            let icon_name = if is_running {
                                "media-playback-stop-symbolic"
                            } else {
                                "media-playback-start-symbolic"
                            };
                            let action_cmd = if is_running { "stop" } else { "start" };
                            let tooltip = if is_running {
                                "Stop Container"
                            } else {
                                "Start Container"
                            };

                            let toggle_btn = gtk4::Button::from_icon_name(icon_name);
                            toggle_btn.add_css_class("flat");
                            toggle_btn.set_tooltip_text(Some(tooltip));

                            let h_toggle = h.clone();
                            let p_toggle = p.clone();
                            let n_toggle = item_name.clone();
                            let rb_toggle = rb_inner.clone();
                            let cmd_str = action_cmd.to_string();

                            toggle_btn.connect_clicked(move |_| {
                                let h_c = h_toggle.clone();
                                let p_c = p_toggle.clone();
                                let n_c = n_toggle.clone();
                                let rb_c = rb_toggle.clone();
                                let c_str = cmd_str.clone();
                                glib::MainContext::default().spawn_local(async move {
                                    let _ =
                                        perform_docker_action(&h_c, p_c.as_deref(), &c_str, &n_c)
                                            .await;
                                    rb_c.emit_clicked();
                                });
                            });
                            actions.append(&toggle_btn);
                        }

                        let delete_btn = gtk4::Button::from_icon_name("user-trash-symbolic");
                        delete_btn.add_css_class("flat");
                        delete_btn.add_css_class("error");
                        delete_btn.set_tooltip_text(Some(if is_container {
                            "Remove Container"
                        } else {
                            "Remove Image"
                        }));

                        let h_del = h.clone();
                        let p_del = p.clone();
                        let n_del = item_name.clone();
                        let rb_del = rb_inner.clone();
                        let is_c_del = is_container;
                        delete_btn.connect_clicked(move |_| {
                            let h_c = h_del.clone();
                            let p_c = p_del.clone();
                            let n_c = n_del.clone();
                            let rb_c = rb_del.clone();
                            glib::MainContext::default().spawn_local(async move {
                                let sub_cmd = if is_c_del { "rm -f" } else { "rmi" };
                                let _ = perform_docker_action(&h_c, p_c.as_deref(), sub_cmd, &n_c)
                                    .await;
                                rb_c.emit_clicked();
                            });
                        });

                        actions.append(&delete_btn);
                        row.append(&actions);
                        lb.append(&row);
                    }
                }
            });
        });

        box_
    }

    fn build_stat_card(&self, title: &str, value: &str, icon_name: Option<&str>) -> gtk4::Box {
        let card = gtk4::Box::new(gtk4::Orientation::Vertical, 12);
        card.set_width_request(160);
        card.add_css_class("docker-card");

        let header = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        let icon_img = match icon_name {
            Some("container") => crate::ui::get_container_icon(),
            Some(name) => gtk4::Image::from_icon_name(name),
            None => crate::ui::get_docker_icon(),
        };
        let label_title = gtk4::Label::builder()
            .label(title)
            .css_classes(vec!["dim-label".to_string(), "bold".to_string()])
            .build();
        header.append(&icon_img);
        header.append(&label_title);

        let label_value = gtk4::Label::builder()
            .label(value)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["stat-value".to_string()])
            .build();
        label_value.set_widget_name(&format!("stat_value_{}", title.to_lowercase()));

        card.append(&header);
        card.append(&label_value);
        card
    }

    fn refresh_dashboard(&self) {
        let host = self.host.clone();
        let password = self.password.clone();
        let stack = self.stack.clone();

        glib::MainContext::default().spawn_local(async move {
            let result = get_docker_stats(&host, password.as_deref()).await;

            match result {
                Ok(parts) => {
                    if parts.len() >= 4 {
                        if let Some(dashboard) = stack.child_by_name("dashboard") {
                            Self::update_stat(&dashboard, "stat_value_containers", &parts[0]);
                            Self::update_stat(&dashboard, "stat_value_images", &parts[1]);
                            Self::update_stat(&dashboard, "stat_value_running", &parts[2]);
                            Self::update_stat(&dashboard, "stat_value_paused", &parts[3]);

                            if stack.visible_child_name() == Some(glib::GString::from("loading")) {
                                stack.set_visible_child_name("dashboard");
                            }
                        }
                    }
                }
                _ => {}
            }
        });
    }

    fn update_stat(root: &gtk4::Widget, name: &str, value: &str) {
        if let Some(label) = Self::find_child_by_name(root, name) {
            if let Some(label) = label.downcast_ref::<gtk4::Label>() {
                label.set_text(value);
            }
        }
    }

    fn find_child_by_name(widget: &gtk4::Widget, name: &str) -> Option<gtk4::Widget> {
        if widget.widget_name() == name {
            return Some(widget.clone());
        }
        let mut next = widget.first_child();
        while let Some(child) = next {
            if let Some(found) = Self::find_child_by_name(&child, name) {
                return Some(found);
            }
            next = child.next_sibling();
        }
        None
    }
}
