use gtk4::prelude::*;
use crate::config_observer::{SshHost, load_hosts};

pub struct ServerList {
    pub container: gtk4::ScrolledWindow,
    pub flow_box: gtk4::FlowBox,
}

impl ServerList {
    pub fn new<F>(on_connect: F) -> Self 
    where F: Fn(&SshHost) + 'static + Clone
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
        sl.refresh(on_connect);
        sl
    }

    pub fn refresh<F>(&self, on_connect: F) 
    where F: Fn(&SshHost) + 'static + Clone
    {
        while let Some(child) = self.flow_box.first_child() {
            self.flow_box.remove(&child);
        }

        let hosts = load_hosts();
        for host in hosts {
            self.add_host_row(&host, on_connect.clone());
        }
    }

    fn add_host_row<F>(&self, host: &SshHost, on_connect: F) 
    where F: Fn(&SshHost) + 'static
    {
        let frame = gtk4::Frame::new(None);
        frame.add_css_class("card");

        let row_box = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        row_box.set_margin_top(12);
        row_box.set_margin_bottom(12);
        row_box.set_margin_start(12);
        row_box.set_margin_end(12);

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

        row_box.append(&alias_label);
        row_box.append(&host_label);

        let gesture = gtk4::GestureClick::new();
        let host_clone = host.clone();
        gesture.connect_released(move |_, _, _, _| {
            on_connect(&host_clone);
        });
        frame.add_controller(gesture);

        frame.set_child(Some(&row_box));
        self.flow_box.append(&frame);
    }
}
