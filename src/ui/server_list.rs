use crate::config_observer::{SshHost, load_hosts};
use gtk4::glib;
use gtk4::prelude::*;

pub enum ServerAction {
    Connect(SshHost, Option<String>),
    Edit(SshHost),
    Delete(SshHost),
}

pub struct ServerList {
    pub container: gtk4::ScrolledWindow,
    pub flow_box: gtk4::FlowBox,
}

impl ServerList {
    pub fn new<F>(on_action: F) -> Self
    where
        F: Fn(ServerAction) + 'static + Clone,
    {
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();
        let flow_box = gtk4::FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .valign(gtk4::Align::Start)
            .max_children_per_line(3)
            .min_children_per_line(1)
            .column_spacing(16)
            .row_spacing(16)
            .build();
        flow_box.add_css_class("page");
        scrolled.set_child(Some(&flow_box));

        let sl = Self {
            container: scrolled,
            flow_box,
        };
        sl.refresh(on_action);
        sl
    }

    pub fn refresh<F>(&self, on_action: F)
    where
        F: Fn(ServerAction) + 'static + Clone,
    {
        while let Some(child) = self.flow_box.first_child() {
            self.flow_box.remove(&child);
        }

        let hosts = load_hosts().unwrap_or_else(|e| {
            tracing::error!("Failed to load hosts: {}", e);
            Vec::new()
        });
        for host in hosts {
            self.add_host_row(&host, on_action.clone());
        }
    }

    fn add_host_row<F>(&self, host: &SshHost, on_action: F)
    where
        F: Fn(ServerAction) + 'static + Clone,
    {
        let frame = gtk4::Frame::new(None);
        frame.add_css_class("card");
        frame.set_width_request(260);
        crate::ui::set_pointer_cursor(&frame);

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 10);
        crate::ui::set_pointer_cursor(&content_box);

        let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        let icon = gtk4::Image::from_icon_name("computer-symbolic");
        icon.set_pixel_size(20);
        icon.add_css_class("server-card-icon");

        let title_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        title_box.set_hexpand(true);
        let alias_label = gtk4::Label::builder()
            .label(&host.alias)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["heading".to_string()])
            .build();
        let host_info = format!("{}@{}", host.user.as_deref().unwrap_or("root"), host.hostname);
        let host_label = gtk4::Label::builder()
            .label(&host_info)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["dim-label".to_string(), "caption".to_string()])
            .build();
        title_box.append(&alias_label);
        title_box.append(&host_label);

        let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 2);
        actions_box.add_css_class("server-card-actions");
        let edit_btn = gtk4::Button::from_icon_name("document-edit-symbolic");
        edit_btn.add_css_class("flat");
        let host_edit = host.clone();
        let on_action_edit = on_action.clone();
        edit_btn.connect_clicked(move |_| {
            on_action_edit(ServerAction::Edit(host_edit.clone()));
        });

        let delete_btn = gtk4::Button::from_icon_name("user-trash-symbolic");
        delete_btn.add_css_class("flat");
        delete_btn.add_css_class("error");
        let host_del = host.clone();
        let on_action_del = on_action.clone();
        delete_btn.connect_clicked(move |_| {
            on_action_del(ServerAction::Delete(host_del.clone()));
        });

        actions_box.append(&edit_btn);
        actions_box.append(&delete_btn);

        header_box.append(&icon);
        header_box.append(&title_box);
        header_box.append(&actions_box);
        content_box.append(&header_box);

        let gesture = gtk4::GestureClick::new();
        let host_conn = host.clone();
        let on_action_conn = on_action.clone();
        gesture.connect_released(move |_, _, _, _| {
            let h = host_conn.clone();
            let oa = on_action_conn.clone();
            glib::MainContext::default().spawn_local(async move {
                let password = crate::config_observer::get_keyring_password(&h.alias).await;
                oa(ServerAction::Connect(h, password));
            });
        });
        frame.add_controller(gesture);

        frame.set_child(Some(&content_box));
        self.flow_box.insert(&frame, -1);
    }
}
