use gtk4::prelude::*;
use crate::config_observer::SshHost;
use nucleo_matcher::{Matcher, Config, Utf32String};

pub struct Hud {
    pub popover: gtk4::Popover,
    pub entry: gtk4::Entry,
    pub list_box: gtk4::ListBox,
    matcher: Matcher,
}

impl Hud {
    pub fn new() -> Self {
        let popover = gtk4::Popover::new();
        let box_container = gtk4::Box::new(gtk4::Orientation::Vertical, 6);
        box_container.set_margin_top(6);
        box_container.set_margin_bottom(6);
        box_container.set_margin_start(6);
        box_container.set_margin_end(6);

        let entry = gtk4::Entry::new();
        entry.set_placeholder_text(Some("Search hosts..."));
        box_container.append(&entry);

        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_min_content_height(300);
        scrolled.set_min_content_width(400);
        
        let list_box = gtk4::ListBox::new();
        scrolled.set_child(Some(&list_box));
        box_container.append(&scrolled);

        popover.set_child(Some(&box_container));

        Self {
            popover,
            entry,
            list_box,
            matcher: Matcher::new(Config::DEFAULT),
        }
    }

    pub fn update_results(&self, hosts: &[SshHost], query: &str) {
        // Clear existing results
        while let Some(row) = self.list_box.first_child() {
            self.list_box.remove(&row);
        }

        if query.is_empty() {
            for host in hosts.iter().take(10) {
                self.add_row(host);
            }
            return;
        }

        let mut matches: Vec<(u16, &SshHost)> = Vec::new();
        let query_utf32 = Utf32String::from(query);

        let mut matcher = self.matcher.clone();

        for host in hosts {
            let text = format!("{} {}", host.alias, host.hostname);
            let text_utf32 = Utf32String::from(text.as_str());
            
            if let Some(score) = matcher.fuzzy_match(text_utf32.slice(..), query_utf32.slice(..)) {
                matches.push((score, host));
            }
        }

        matches.sort_by(|a, b| b.0.cmp(&a.0));

        for (_, host) in matches.iter().take(10) {
            self.add_row(host);
        }
    }

    fn add_row(&self, host: &SshHost) {
        let row_box = gtk4::Box::new(gtk4::Orientation::Vertical, 2);
        let alias_label = gtk4::Label::new(Some(&host.alias));
        alias_label.set_halign(gtk4::Align::Start);
        alias_label.add_css_class("heading");
        
        let host_label = gtk4::Label::new(Some(&format!("{}@{}", host.user.as_deref().unwrap_or(""), host.hostname)));
        host_label.set_halign(gtk4::Align::Start);
        host_label.add_css_class("caption");

        row_box.append(&alias_label);
        row_box.append(&host_label);
        
        self.list_box.append(&row_box);
    }
}
