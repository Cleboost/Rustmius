use gtk4::prelude::*;
use crate::config_observer::{SshHost, load_hosts};

pub enum ServerAction {
    Connect(SshHost),
    Edit(SshHost),
    Delete(SshHost),
}

pub struct ServerList {
    pub container: gtk4::ScrolledWindow,
    pub flow_box: gtk4::FlowBox,
}

impl ServerList {
    pub fn new<F>(on_action: F) -> Self 
    where F: Fn(ServerAction) + 'static + Clone
    {
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();
        
        let flow_box = gtk4::FlowBox::builder()
            .selection_mode(gtk4::SelectionMode::None)
            .valign(gtk4::Align::Start)
            .max_children_per_line(4)
            .min_children_per_line(1)
            .column_spacing(12)
            .row_spacing(12)
            .margin_top(24)
            .margin_bottom(24)
            .margin_start(24)
            .margin_end(24)
            .build();
        
        scrolled.set_child(Some(&flow_box));

        let sl = Self { container: scrolled, flow_box };
        sl.refresh(on_action);
        sl
    }

    pub fn refresh<F>(&self, on_action: F) 
    where F: Fn(ServerAction) + 'static + Clone
    {
        while let Some(child) = self.flow_box.first_child() {
            self.flow_box.remove(&child);
        }

        let hosts = load_hosts();
        for host in hosts {
            self.add_host_row(&host, on_action.clone());
        }
    }

    fn add_host_row<F>(&self, host: &SshHost, on_action: F) 
    where F: Fn(ServerAction) + 'static
    {
        let frame = gtk4::Frame::new(None);
        frame.add_css_class("card");

        let content_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        content_box.set_margin_top(12);
        content_box.set_margin_bottom(12);
        content_box.set_margin_start(12);
        content_box.set_margin_end(12);

        // Header with Alias and Actions
        let header_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 8);
        
        let alias_label = gtk4::Label::builder()
            .label(&host.alias)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(vec!["heading".to_string()])
            .build();

        let actions_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 4);
        
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

        header_box.append(&alias_label);
        header_box.append(&actions_box);

        let host_info = format!("{}@{}", host.user.as_deref().unwrap_or("root"), host.hostname);
        let host_label = gtk4::Label::builder()
            .label(&host_info)
            .halign(gtk4::Align::Start)
            .css_classes(vec!["dim-label".to_string(), "caption".to_string()])
            .build();

        content_box.append(&header_box);
        content_box.append(&host_label);

        let gesture = gtk4::GestureClick::new();
        let host_conn = host.clone();
        let on_action_conn = on_action.clone();
        gesture.connect_released(move |_, _, _, _| {
            on_action_conn(ServerAction::Connect(host_conn.clone()));
        });
        frame.add_controller(gesture);

        frame.set_child(Some(&content_box));
        self.flow_box.insert(&frame, -1);
    }
}
