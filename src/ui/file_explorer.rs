#![allow(deprecated)]
use gtk4::prelude::*;
use gtk4::{glib, gio, gdk};
use crate::config_observer::SshHost;
use crate::sftp_engine::{list_files, delete_file, rename_file, create_dir, create_file, upload_file, download_file_sync, RemoteFile};
use std::rc::Rc;
use std::cell::RefCell;

pub struct FileExplorer {
    pub container: gtk4::Box,
    list_box: gtk4::ListBox,
    current_path: Rc<RefCell<String>>,
    path_entry: gtk4::Entry,
    status_label: gtk4::Label,
    host: SshHost,
    password: Option<String>,
    
    files: Rc<RefCell<Vec<RemoteFile>>>,
}

impl FileExplorer {
    pub fn new(host: SshHost, password: Option<String>) -> Self {
        let provider = gtk4::CssProvider::new();
        provider.load_from_string("
            .sftp-list row { padding: 6px 12px; }
            .sftp-list row:selected { background-color: @theme_selected_bg_color; color: @theme_selected_fg_color; }
            .sftp-list row:hover:not(:selected) { background-color: alpha(@theme_fg_color, 0.04); }
            .sftp-status { padding: 4px 12px; background-color: alpha(@theme_fg_color, 0.03); border-top: 1px solid alpha(@theme_fg_color, 0.08); }
            .sftp-path { font-family: monospace; font-size: 0.9em; }
            .file-size { opacity: 0.55; font-size: 0.85em; font-family: monospace; }
        ");
        gtk4::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        let user = host.user.as_deref().unwrap_or("root");
        let initial_path = if user == "root" {
            "/root/".to_string()
        } else {
            format!("/home/{}/", user)
        };

        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
        let current_path = Rc::new(RefCell::new(initial_path.clone()));
        let files: Rc<RefCell<Vec<RemoteFile>>> = Rc::new(RefCell::new(Vec::new()));

        let path_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
        path_bar.set_margin_top(6); path_bar.set_margin_bottom(6);
        path_bar.set_margin_start(8); path_bar.set_margin_end(8);

        let back_btn = gtk4::Button::from_icon_name("go-up-symbolic");
        back_btn.set_tooltip_text(Some("Parent directory"));
        back_btn.add_css_class("flat");
        let refresh_btn = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh_btn.set_tooltip_text(Some("Refresh"));
        refresh_btn.add_css_class("flat");
        let path_entry = gtk4::Entry::new();
        path_entry.set_hexpand(true);
        path_entry.set_text(&initial_path);
        path_entry.add_css_class("sftp-path");
        let new_folder_btn = gtk4::Button::from_icon_name("folder-new-symbolic");
        new_folder_btn.set_tooltip_text(Some("New folder"));
        new_folder_btn.add_css_class("flat");

        path_bar.append(&back_btn);
        path_bar.append(&path_entry);
        path_bar.append(&new_folder_btn);
        path_bar.append(&refresh_btn);
        container.append(&path_bar);

        let scrolled = gtk4::ScrolledWindow::builder().vexpand(true).build();
        let list_box = gtk4::ListBox::new();
        list_box.set_selection_mode(gtk4::SelectionMode::Single);
        list_box.add_css_class("sftp-list");
        list_box.add_css_class("boxed-list");
        scrolled.set_child(Some(&list_box));
        container.append(&scrolled);

        let status_label = gtk4::Label::builder()
            .label("Ready")
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .css_classes(vec!["caption".to_string(), "dim-label".to_string()])
            .build();
        let status_bar = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);
        status_bar.add_css_class("sftp-status");
        status_bar.append(&status_label);
        container.append(&status_bar);

        let explorer = Self {
            container: container.clone(),
            list_box: list_box.clone(),
            current_path: current_path.clone(),
            path_entry: path_entry.clone(),
            status_label: status_label.clone(),
            host: host.clone(),
            password: password.clone(),
            files: files.clone(),
        };

        let formats = gdk::ContentFormats::builder()
            .add_type(gdk::FileList::static_type())
            .add_type(glib::Bytes::static_type())
            .add_type(glib::Type::STRING)
            .add_mime_type("text/uri-list")
            .add_mime_type("text/plain")
            .build();

        let drop_actions = gdk::DragAction::COPY.union(gdk::DragAction::MOVE);
        let drop_target = gtk4::DropTarget::builder()
            .actions(drop_actions)
            .formats(&formats)
            .preload(true)
            .build();
        drop_target.set_propagation_phase(gtk4::PropagationPhase::Capture);

        let h_enter = explorer.clone_handle();
        drop_target.connect_enter(move |_, _, _| {
            h_enter.status_label.set_text("Drop here to upload...");
            drop_actions
        });
        let h_leave = explorer.clone_handle();
        drop_target.connect_leave(move |_| {
            h_leave.status_label.set_text("Ready");
        });

        let h_drop = explorer.clone_handle();
        drop_target.connect_drop(move |_, value, _, _| {
            println!("Drop event received!");
            let h = h_drop.clone();
            let remote_dir = h.current_path.borrow().clone();
            let mut paths: Vec<std::path::PathBuf> = Vec::new();

            if let Ok(file_list) = value.get::<gdk::FileList>() {
                for file in file_list.files() {
                    if let Some(p) = file.path() { paths.push(p); }
                }
            }

            if paths.is_empty()
                && let Ok(bytes) = value.get::<glib::Bytes>()
                    && let Ok(uris_str) = std::str::from_utf8(bytes.as_ref()) {
                        paths.extend(parse_uri_list_paths(uris_str));
                    }
            
            if paths.is_empty()
                && let Ok(uris_str) = value.get::<String>() {
                    paths.extend(parse_uri_list_paths(&uris_str));
                }

            if paths.is_empty() {
                h.status_label.set_text("Drop: no valid files found.");
                return false;
            }

            let count = paths.len();
            println!("Starting upload of {} files", count);
            h.status_label.set_text(&format!("Uploading {} file(s)...", count));

            for local_path in paths {
                let filename = local_path.file_name().unwrap_or_default().to_string_lossy().to_string();
                let remote_dest = format!("{}{}", remote_dir, filename);
                let local_str = local_path.to_string_lossy().to_string();
                let h_task = h.clone();
                glib::MainContext::default().spawn_local(async move {
                    match upload_file(h_task.host.clone(), h_task.password.clone(), local_str, remote_dest).await {
                        Ok(_) => {
                            h_task.status_label.set_text("Upload complete.");
                            h_task.refresh();
                        }
                        Err(e) => {
                            h_task.status_label.set_text(&format!("Upload error: {}", e));
                        }
                    }
                });
            }
            true
        });
        list_box.add_controller(drop_target);

        let h_activate = explorer.clone_handle();
        let files_activate = files.clone();
        list_box.connect_row_activated(move |_, row| {
            let idx = row.index() as usize;
            let files_ref = files_activate.borrow();
            if let Some(f) = files_ref.get(idx)
                && f.is_dir {
                    let mut path = h_activate.current_path.borrow_mut();
                    if !path.ends_with('/') { path.push('/'); }
                    path.push_str(&f.name); path.push('/');
                    h_activate.path_entry.set_text(&path);
                    drop(path);
                    drop(files_ref);
                    h_activate.refresh();
                }
        });

        let sl_back = explorer.clone_handle();
        let pe_back = path_entry.clone();
        back_btn.connect_clicked(move |_| {
            let mut path = sl_back.current_path.borrow_mut();
            if *path != "/" {
                let p = std::path::Path::new(&*path);
                if let Some(parent) = p.parent() {
                    let mut new_p = parent.to_string_lossy().to_string();
                    if !new_p.ends_with('/') { new_p.push('/'); }
                    *path = new_p.clone(); pe_back.set_text(&new_p);
                    drop(path); sl_back.refresh();
                }
            }
        });

        let sl_enter = explorer.clone_handle();
        path_entry.connect_activate(move |e| {
            let mut path = e.text().to_string();
            if !path.starts_with('/') { path = format!("/{}", path); }
            if !path.ends_with('/') { path.push('/'); }
            *sl_enter.current_path.borrow_mut() = path;
            sl_enter.refresh();
        });

        let sl_refresh = explorer.clone_handle();
        refresh_btn.connect_clicked(move |_| { sl_refresh.refresh(); });

        let sl_new_btn = explorer.clone_handle();
        new_folder_btn.connect_clicked(move |_| {
            let sl = sl_new_btn.clone(); let cur = sl.current_path.borrow().clone();
            let parent_window = sl.list_box.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
            show_input_dialog(parent_window.as_ref(), "New Folder", "Name:", "", move |n| {
                let sli = sl.clone(); let p = format!("{}{}", cur, n);
                glib::MainContext::default().spawn_local(async move {
                    if let Ok(_) = create_dir(sli.host.clone(), sli.password.clone(), p).await { sli.refresh(); }
                });
            });
        });

        explorer.refresh();
        explorer
    }

    fn clone_handle(&self) -> ExplorerHandle {
        ExplorerHandle {
            list_box: self.list_box.clone(),
            current_path: self.current_path.clone(),
            path_entry: self.path_entry.clone(),
            status_label: self.status_label.clone(),
            host: self.host.clone(),
            password: self.password.clone(),
            files: self.files.clone(),
        }
    }

    pub fn refresh(&self) {
        self.clone_handle().refresh();
    }
}

#[derive(Clone)]
struct ExplorerHandle {
    list_box: gtk4::ListBox,
    current_path: Rc<RefCell<String>>,
    path_entry: gtk4::Entry,
    status_label: gtk4::Label,
    host: SshHost,
    password: Option<String>,
    files: Rc<RefCell<Vec<RemoteFile>>>,
}

impl ExplorerHandle {
    fn refresh(&self) {
        let lb = self.list_box.clone();
        let h = self.host.clone();
        let pw = self.password.clone();
        let p = self.current_path.borrow().clone();
        let handle = self.clone();

        while let Some(row) = lb.row_at_index(0) {
            unparent_children(&row);
            lb.remove(&row);
        }
        let loading = gtk4::Label::builder().label("Loading...").margin_top(12).margin_bottom(12).build();
        lb.append(&loading);

        glib::MainContext::default().spawn_local(async move {
            match list_files(h, pw, p).await {
                Ok(files) => {
                    while let Some(row) = lb.row_at_index(0) {
                        unparent_children(&row);
                        lb.remove(&row);
                    }
                    *handle.files.borrow_mut() = files.clone();
                    let count = files.len();
                    for f in files {
                        handle.add_file_row(f);
                    }
                    handle.status_label.set_text(&format!("{} items", count));
                },
                Err(e) => {
                    while let Some(row) = lb.row_at_index(0) { lb.remove(&row); }
                    let err_label = gtk4::Label::new(Some(&format!("Error: {}", e)));
                    err_label.set_margin_top(12);
                    lb.append(&err_label);
                    handle.status_label.set_text(&format!("Error: {}", e));
                }
            }
        });
    }

    fn add_file_row(&self, file: RemoteFile) {
        let row_content = gtk4::Box::new(gtk4::Orientation::Horizontal, 10);
        row_content.set_margin_top(4); row_content.set_margin_bottom(4);

        let theme = gtk4::IconTheme::for_display(&gdk::Display::default().unwrap());
        let paintable = theme.lookup_by_gicon(
            &file_icon(&file),
            32,
            1,
            gtk4::TextDirection::None,
            gtk4::IconLookupFlags::FORCE_REGULAR,
        );
        let icon = gtk4::Image::from_paintable(Some(&paintable));
        icon.set_pixel_size(32);
        row_content.append(&icon);

        let name_label = gtk4::Label::builder()
            .label(&file.name)
            .halign(gtk4::Align::Start)
            .hexpand(true)
            .ellipsize(gtk4::pango::EllipsizeMode::Middle)
            .build();
        row_content.append(&name_label);

        if !file.is_dir {
            let size_str = format_file_size(file.size);
            let size_label = gtk4::Label::builder()
                .label(&size_str)
                .halign(gtk4::Align::End)
                .build();
            size_label.add_css_class("file-size");
            row_content.append(&size_label);
        }

        if !file.is_dir {
            let source = gtk4::DragSource::new();
            source.set_actions(gdk::DragAction::COPY);
            let h_drag = self.clone();
            let f_drag = file.clone();

            source.connect_prepare(move |src, _, _| {
                let h = h_drag.clone();
                let f = f_drag.clone();
                let remote_path = format!("{}{}", h.current_path.borrow(), f.name);
                let local_tmp = format!("/tmp/rustmius_dnd_{}", f.name);

                let paintable = gtk4::IconTheme::for_display(&gdk::Display::default().unwrap())
                    .lookup_by_gicon(
                        &file_icon(&f),
                        32,
                        1,
                        gtk4::TextDirection::None,
                        gtk4::IconLookupFlags::FORCE_REGULAR,
                    );
                src.set_icon(Some(&paintable), 16, 16);

                h.status_label.set_text(&format!("Downloading {}...", f.name));

                let host = h.host.clone();
                let password = h.password.clone();
                let rp = remote_path.clone();
                let lp = local_tmp.clone();

                h.status_label.set_text(&format!("Preparing {}...", f.name));
                std::thread::spawn(move || {
                    let _ = download_file_sync(host, password, rp, lp);
                });

                let uri = format!("file://{}\r\n", local_tmp);
                let bytes = glib::Bytes::from(uri.as_bytes());
                Some(gdk::ContentProvider::for_bytes("text/uri-list", &bytes))
            });

            let h_end = self.clone();
            source.connect_drag_end(move |_, _, _| {
                h_end.status_label.set_text("Ready");
            });

            row_content.add_controller(source);
        }

        let gesture = gtk4::GestureClick::builder().button(3).build();
        let h_menu = self.clone();
        let f_menu = file.clone();
        gesture.connect_released(move |gesture_self, _, x, y| {
            let h = h_menu.clone();
            let f = f_menu.clone();
            let menu = gio::Menu::new();
            menu.append(Some("Rename"), Some("row.rename"));
            menu.append(Some("Delete"), Some("row.delete"));
            if f.is_dir {
                menu.append(Some("New File"), Some("row.new_file"));
                menu.append(Some("New Folder"), Some("row.new_folder"));
            }

            let group = gio::SimpleActionGroup::new();

            let h_del = h.clone(); let f_del = f.clone();
            let del_action = gio::SimpleAction::new("delete", None);
            del_action.connect_activate(move |_, _| {
                let hi = h_del.clone(); let fi = f_del.clone();
                let path = format!("{}{}", hi.current_path.borrow(), fi.name);
                let parent_window = hi.list_box.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                show_confirm_dialog(parent_window.as_ref(), "Confirm Delete", &format!("Delete '{}'?", fi.name), move || {
                    let hii = hi.clone(); let p = path.clone(); let isd = fi.is_dir;
                    glib::MainContext::default().spawn_local(async move {
                        match delete_file(hii.host.clone(), hii.password.clone(), p, isd).await {
                            Ok(_) => hii.refresh(),
                            Err(e) => hii.status_label.set_text(&format!("Delete error: {}", e)),
                        }
                    });
                });
            });
            group.add_action(&del_action);

            let h_ren = h.clone(); let f_ren = f.clone();
            let ren_action = gio::SimpleAction::new("rename", None);
            ren_action.connect_activate(move |_, _| {
                let hi = h_ren.clone(); let fi = f_ren.clone();
                let cur = hi.current_path.borrow().clone();
                let old_name = fi.name.clone();
                let old_name_cloned = old_name.clone();
                let parent_window = hi.list_box.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                show_input_dialog(parent_window.as_ref(), "Rename", "New name:", &old_name, move |new_n| {
                    let hii = hi.clone();
                    let old_p = format!("{}{}", cur, old_name_cloned);
                    let new_p = format!("{}{}", cur, new_n);
                    glib::MainContext::default().spawn_local(async move {
                        match rename_file(hii.host.clone(), hii.password.clone(), old_p, new_p).await {
                            Ok(_) => hii.refresh(),
                            Err(e) => hii.status_label.set_text(&format!("Rename error: {}", e)),
                        }
                    });
                });
            });
            group.add_action(&ren_action);

            if f.is_dir {
                
                let h_nf = h.clone(); let f_nf = f.clone();
                let nf_action = gio::SimpleAction::new("new_file", None);
                nf_action.connect_activate(move |_, _| {
                    let hi = h_nf.clone(); let fi = f_nf.clone();
                    let pb = format!("{}{}/", hi.current_path.borrow(), fi.name);
                    let parent_window = hi.list_box.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                    show_input_dialog(parent_window.as_ref(), "New File", "Name:", "", move |n| {
                        let hii = hi.clone(); let p = format!("{}{}", pb, n);
                        glib::MainContext::default().spawn_local(async move {
                            match create_file(hii.host.clone(), hii.password.clone(), p).await {
                                Ok(_) => hii.refresh(),
                                Err(e) => hii.status_label.set_text(&format!("Error: {}", e)),
                            }
                        });
                    });
                });
                group.add_action(&nf_action);

                let h_nd = h.clone(); let f_nd = f.clone();
                let nd_action = gio::SimpleAction::new("new_folder", None);
                nd_action.connect_activate(move |_, _| {
                    let hi = h_nd.clone(); let f_i = f_nd.clone();
                    let pb = format!("{}{}/", hi.current_path.borrow(), f_i.name);
                    let parent_window = hi.list_box.root().and_then(|r| r.downcast::<gtk4::Window>().ok());
                    show_input_dialog(parent_window.as_ref(), "New Folder", "Name:", "", move |n| {
                        let hii = hi.clone(); let p = format!("{}{}", pb, n);
                        glib::MainContext::default().spawn_local(async move {
                            match create_dir(hii.host.clone(), hii.password.clone(), p).await {
                                Ok(_) => hii.refresh(),
                                Err(e) => hii.status_label.set_text(&format!("Error: {}", e)),
                            }
                        });
                    });
                });
                group.add_action(&nd_action);
            }

            if let Some(widget) = gesture_self.widget()
                && let Some(parent_row) = row_content_ancestor::<gtk4::ListBoxRow>(&widget) {
                    let popover = gtk4::PopoverMenu::builder().menu_model(&menu).has_arrow(false).build();
                    popover.insert_action_group("row", Some(&group));
                    popover.set_parent(&parent_row);
                    popover.set_pointing_to(Some(&gdk::Rectangle::new(x as i32, y as i32, 1, 1)));
                    popover.popup();
                }
        });
        row_content.add_controller(gesture);

        self.list_box.append(&row_content);
    }
}

fn parse_uri_list_paths(uris_str: &str) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    for uri in uris_str.split(['\n', '\r', '\0']).map(str::trim) {
        if uri.is_empty() || uri.starts_with('#') {
            continue;
        }
        let file = gio::File::for_uri(uri);
        if let Some(p) = file.path() {
            paths.push(p);
        }
    }
    paths
}

fn unparent_children(widget: &impl IsA<gtk4::Widget>) {
    let mut child = widget.first_child();
    while let Some(c) = child {
        let next = c.next_sibling();
        if c.is::<gtk4::Popover>() || c.is::<gtk4::PopoverMenu>() {
            c.unparent();
        }
        child = next;
    }
}

fn row_content_ancestor<T: IsA<gtk4::Widget>>(widget: &gtk4::Widget) -> Option<T> {
    let mut current = widget.parent();
    while let Some(w) = current {
        if let Ok(typed) = w.clone().downcast::<T>() {
            return Some(typed);
        }
        current = w.parent();
    }
    None
}

fn file_icon(file: &RemoteFile) -> gio::Icon {
    let content_type = if file.is_dir {
        "inode/directory".into()
    } else {
        let (guess, _) = gio::content_type_guess(Some(&file.name), None::<&[u8]>);
        guess
    };

    gio::content_type_get_icon(&content_type)
}

fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = 1024 * 1024;
    const GB: u64 = 1024 * 1024 * 1024;
    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

fn show_input_dialog<F>(parent: Option<&gtk4::Window>, title: &str, label: &str, initial: &str, on_submit: F)
where F: Fn(String) + 'static
{
    let dialog = gtk4::Dialog::new();
    dialog.set_title(Some(title));
    dialog.set_modal(true);
    if let Some(p) = parent {
        dialog.set_transient_for(Some(p));
    }
    
    let content = dialog.content_area();
    content.set_margin_top(12); content.set_margin_bottom(12);
    content.set_margin_start(12); content.set_margin_end(12);
    content.set_spacing(12);
    content.append(&gtk4::Label::new(Some(label)));
    
    let entry = gtk4::Entry::builder().text(initial).build();
    content.append(&entry);
    
    dialog.add_button("Cancel", gtk4::ResponseType::Cancel);
    dialog.add_button("OK", gtk4::ResponseType::Ok);
    
    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Ok {
            let text = entry.text().to_string();
            if !text.is_empty() { on_submit(text); }
        }
        d.close();
    });
    dialog.present();
}

fn show_confirm_dialog<F>(parent: Option<&gtk4::Window>, title: &str, message: &str, on_confirm: F)
where F: Fn() + 'static
{
    let dialog = gtk4::MessageDialog::new(
        parent,
        gtk4::DialogFlags::MODAL,
        gtk4::MessageType::Question,
        gtk4::ButtonsType::YesNo,
        message
    );
    dialog.set_title(Some(title));
    dialog.connect_response(move |d, res| {
        if res == gtk4::ResponseType::Yes { on_confirm(); }
        d.close();
    });
    dialog.present();
}
