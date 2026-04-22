use gtk4::prelude::*;
use crate::config_observer::SshHost;
use crate::ssh_engine::run_remote_command;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;
use gtk4::glib;

pub struct SystemMonitor {
    pub container: gtk4::ScrolledWindow,
}

struct MonitorState {
    current_cpu: f64,
    target_cpu: f64,
    current_ram: f64,
    target_ram: f64,
    current_disk: f64,
    target_disk: f64,
    ram_used: String,
    ram_total: String,
    disk_used: String,
    disk_total: String,
    kernel: String,
    os: String,
    uptime: String,
    cpu_model: String,
    cpu_cores: String,
    arch: String,
    hostname: String,
    ips: String,
}

impl SystemMonitor {
    pub fn new(host: SshHost, password: Option<String>) -> Self {
        let scrolled = gtk4::ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .build();
        
        let container = gtk4::Box::new(gtk4::Orientation::Vertical, 18);
        container.set_margin_top(18);
        container.set_margin_bottom(18);
        container.set_margin_start(18);
        container.set_margin_end(18);
        scrolled.set_child(Some(&container));

        let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 12);
        let refresh_label = gtk4::Label::new(Some("Refresh Rate:"));
        let refresh_dropdown = gtk4::DropDown::from_strings(&["1s", "3s", "5s", "10s"]);
        refresh_dropdown.set_selected(1);
        toolbar.append(&refresh_label);
        toolbar.append(&refresh_dropdown);
        toolbar.set_halign(gtk4::Align::End);
        container.append(&toolbar);

        let sys_frame = gtk4::Frame::new(Some("System Information"));
        let sys_grid = gtk4::Grid::builder()
            .column_spacing(32)
            .row_spacing(12)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .build();
        
        let hostname_label = Self::info_label("Hostname", "Loading...");
        let os_label = Self::info_label("OS", "Loading...");
        let kernel_label = Self::info_label("Kernel", "Loading...");
        let uptime_label = Self::info_label("Uptime", "Loading...");
        let cpu_model_label = Self::info_label("CPU Model", "Loading...");
        let arch_label = Self::info_label("Architecture", "Loading...");

        sys_grid.attach(&hostname_label.0, 0, 0, 1, 1);
        sys_grid.attach(&hostname_label.1, 0, 1, 1, 1);
        sys_grid.attach(&os_label.0, 1, 0, 1, 1);
        sys_grid.attach(&os_label.1, 1, 1, 1, 1);
        sys_grid.attach(&arch_label.0, 2, 0, 1, 1);
        sys_grid.attach(&arch_label.1, 2, 1, 1, 1);
        
        sys_grid.attach(&kernel_label.0, 0, 2, 1, 1);
        sys_grid.attach(&kernel_label.1, 0, 3, 1, 1);
        sys_grid.attach(&uptime_label.0, 1, 2, 1, 1);
        sys_grid.attach(&uptime_label.1, 1, 3, 1, 1);

        sys_grid.attach(&cpu_model_label.0, 0, 4, 3, 1);
        sys_grid.attach(&cpu_model_label.1, 0, 5, 3, 1);

        sys_frame.set_child(Some(&sys_grid));
        container.append(&sys_frame);

        let resource_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 18);
        resource_box.set_homogeneous(true);

        let cpu_card = Self::resource_card("CPU Load", "");
        let ram_card = Self::resource_card("RAM Capacity", "0/0 GB");
        let disk_card = Self::resource_card("Disk Capacity", "0/0 GB");

        resource_box.append(&cpu_card.0);
        resource_box.append(&ram_card.0);
        resource_box.append(&disk_card.0);
        container.append(&resource_box);

        let net_frame = gtk4::Frame::new(Some("Network Interfaces"));
        let ips_label = gtk4::Label::builder()
            .label("Loading...")
            .halign(gtk4::Align::Start)
            .margin_top(12)
            .margin_bottom(12)
            .margin_start(12)
            .margin_end(12)
            .wrap(true)
            .build();
        net_frame.set_child(Some(&ips_label));
        container.append(&net_frame);

        let state = Rc::new(RefCell::new(MonitorState {
            current_cpu: 0.0,
            target_cpu: 0.0,
            current_ram: 0.0,
            target_ram: 0.0,
            current_disk: 0.0,
            target_disk: 0.0,
            ram_used: "0".to_string(),
            ram_total: "0".to_string(),
            disk_used: "0".to_string(),
            disk_total: "0".to_string(),
            kernel: "Loading...".to_string(),
            os: "Loading...".to_string(),
            uptime: "Loading...".to_string(),
            cpu_model: "Loading...".to_string(),
            cpu_cores: "0".to_string(),
            arch: "Loading...".to_string(),
            hostname: "Loading...".to_string(),
            ips: "Loading...".to_string(),
        }));

        let s_cpu = state.clone();
        cpu_card.1.set_draw_func(move |_, cr, w, h| {
            if let Ok(st) = s_cpu.try_borrow() {
                Self::draw_donut(cr, w as f64, h as f64, st.current_cpu);
            }
        });

        let s_ram = state.clone();
        ram_card.1.set_draw_func(move |_, cr, w, h| {
            if let Ok(st) = s_ram.try_borrow() {
                Self::draw_donut(cr, w as f64, h as f64, st.current_ram);
            }
        });

        let s_disk = state.clone();
        disk_card.1.set_draw_func(move |_, cr, w, h| {
            if let Ok(st) = s_disk.try_borrow() {
                Self::draw_donut(cr, w as f64, h as f64, st.current_disk);
            }
        });

        let h_clone = host.clone();
        let p_clone = password.clone();
        let s_clone = state.clone();
        let host_l = hostname_label.1.clone();
        let os_l = os_label.1.clone();
        let ker_l = kernel_label.1.clone();
        let upt_l = uptime_label.1.clone();
        let cpum_l = cpu_model_label.1.clone();
        let arch_l = arch_label.1.clone();
        let ram_detail_l = ram_card.2.clone();
        let disk_detail_l = disk_card.2.clone();
        let ips_l = ips_label.clone();
        let c_draw = cpu_card.1.clone();
        let r_draw = ram_card.1.clone();
        let d_draw = disk_card.1.clone();
        let container_weak = container.downgrade();
        let refresh_drop = refresh_dropdown.clone();

        let s_anim = state.clone();
        let cw_anim = container.downgrade();
        let cd_anim = c_draw.clone();
        let rd_anim = r_draw.clone();
        let dd_anim = d_draw.clone();
        glib::timeout_add_local(Duration::from_millis(32), move || {
            if cw_anim.upgrade().is_none() { return glib::ControlFlow::Break; }
            if let Ok(mut st) = s_anim.try_borrow_mut() {
                let step = 0.1;
                st.current_cpu += (st.target_cpu - st.current_cpu) * step;
                st.current_ram += (st.target_ram - st.current_ram) * step;
                st.current_disk += (st.target_disk - st.current_disk) * step;
                
                cd_anim.queue_draw();
                rd_anim.queue_draw();
                dd_anim.queue_draw();
            }
            glib::ControlFlow::Continue
        });

        glib::MainContext::default().spawn_local(async move {
            loop {
                if container_weak.upgrade().is_none() { break; }
                
                let cmd = "export LC_ALL=C; \
                           echo \"---OS---\"; cat /etc/os-release | grep PRETTY_NAME | cut -d'\"' -f2; \
                           echo \"---KERNEL---\"; uname -r; \
                           echo \"---UPTIME---\"; uptime -p | sed 's/up //'; \
                           echo \"---CPU_MODEL---\"; cat /proc/cpuinfo | grep \"model name\" | head -1 | cut -d':' -f2 | xargs; \
                           echo \"---CPU_CORES---\"; grep -c ^processor /proc/cpuinfo; \
                           echo \"---ARCH---\"; uname -m; \
                           echo \"---HOSTNAME---\"; hostname; \
                           echo \"---IPS---\"; ip -brief addr show | awk '{print $1 \": \" $3}'; \
                           echo \"---RAM---\"; free -h | grep Mem | awk '{print $3 \" / \" $2}'; \
                           echo \"---RAM_P---\"; free | grep Mem | awk '{print $3/$2}'; \
                           echo \"---DISK_ALL---\"; df -h / --output=pcent,used,size | awk 'NR==2 {print $1, $2, $3}'; \
                           echo \"---CPU_P---\"; top -bn2 -d 0.2 | grep \"%Cpu\" | tail -1 | awk -F',' '{for(i=1;i<=NF;i++) if($i ~ /id/) print $i}' | awk '{print 100-$1}'";

                let h = h_clone.clone();
                let p = p_clone.clone();
                
                let result = tokio::task::spawn_blocking(move || {
                    run_remote_command(h, p, cmd)
                }).await;

                if let Ok(Ok(output)) = result {
                    if let Ok(mut st) = s_clone.try_borrow_mut() {
                        let mut current_section = "";
                        let mut ips = Vec::new();

                        for line in output.lines() {
                            if line.starts_with("---") && line.ends_with("---") {
                                current_section = line;
                                continue;
                            }
                            match current_section {
                                "---OS---" => st.os = line.to_string(),
                                "---KERNEL---" => st.kernel = line.to_string(),
                                "---UPTIME---" => st.uptime = line.to_string(),
                                "---CPU_MODEL---" => st.cpu_model = line.to_string(),
                                "---CPU_CORES---" => st.cpu_cores = line.to_string(),
                                "---ARCH---" => st.arch = line.to_string(),
                                "---HOSTNAME---" => st.hostname = line.to_string(),
                                "---IPS---" => if !line.is_empty() { ips.push(line); },
                                "---RAM---" => {
                                    let parts: Vec<&str> = line.split(" / ").collect();
                                    if parts.len() == 2 {
                                        st.ram_used = parts[0].to_string();
                                        st.ram_total = parts[1].to_string();
                                    }
                                },
                                "---RAM_P---" => st.target_ram = line.trim().replace(',', ".").parse().unwrap_or(0.0),
                                "---DISK_ALL---" => {
                                    let parts: Vec<&str> = line.split_whitespace().collect();
                                    if parts.len() == 3 {
                                        st.target_disk = parts[0].trim_end_matches('%').replace(',', ".").parse::<f64>().unwrap_or(0.0) / 100.0;
                                        st.disk_used = parts[1].to_string();
                                        st.disk_total = parts[2].to_string();
                                    }
                                },
                                "---CPU_P---" => st.target_cpu = line.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0) / 100.0,
                                _ => {}
                            }
                        }
                        st.ips = ips.join("\n");
                    }
                }

                if let Ok(st) = s_clone.try_borrow() {
                    host_l.set_label(&st.hostname);
                    os_l.set_label(&st.os);
                    ker_l.set_label(&st.kernel);
                    upt_l.set_label(&st.uptime);
                    cpum_l.set_label(&format!("{} ({} Cores)", st.cpu_model, st.cpu_cores));
                    arch_l.set_label(&st.arch);
                    ram_detail_l.set_label(&format!("{} / {}", st.ram_used, st.ram_total));
                    disk_detail_l.set_label(&format!("{} / {}", st.disk_used, st.disk_total));
                    ips_l.set_label(&st.ips);
                }

                let secs = match refresh_drop.selected() {
                    0 => 1,
                    1 => 3,
                    2 => 5,
                    3 => 10,
                    _ => 3,
                };
                glib::timeout_future(Duration::from_secs(secs)).await;
            }
        });

        Self { container: scrolled }
    }

    fn info_label(title: &str, value: &str) -> (gtk4::Label, gtk4::Label) {
        let t_lbl = gtk4::Label::builder()
            .label(title)
            .halign(gtk4::Align::Start)
            .build();
        t_lbl.add_css_class("caption");
        t_lbl.set_opacity(0.7);

        let v_lbl = gtk4::Label::builder()
            .label(value)
            .halign(gtk4::Align::Start)
            .use_markup(false)
            .build();
        v_lbl.add_css_class("body");
        (t_lbl, v_lbl)
    }

    fn resource_card(title: &str, detail: &str) -> (gtk4::Frame, gtk4::DrawingArea, gtk4::Label) {
        let frame = gtk4::Frame::new(Some(title));
        let bx = gtk4::Box::new(gtk4::Orientation::Vertical, 8);
        bx.set_margin_top(12);
        bx.set_margin_bottom(12);
        bx.set_margin_start(12);
        bx.set_margin_end(12);
        
        let da = gtk4::DrawingArea::builder()
            .content_width(120)
            .content_height(120)
            .build();
        
        let detail_lbl = gtk4::Label::builder()
            .label(detail)
            .halign(gtk4::Align::Center)
            .build();
        detail_lbl.set_opacity(0.8);

        bx.append(&da);
        bx.append(&detail_lbl);
        frame.set_child(Some(&bx));
        (frame, da, detail_lbl)
    }

    fn draw_donut(cr: &gtk4::cairo::Context, width: f64, height: f64, percent: f64) {
        let percent = percent.clamp(0.0, 1.0);
        let center_x = width / 2.0;
        let center_y = height / 2.0;
        let radius = width.min(height) / 2.0 - 10.0;
        let thickness = 14.0;

        cr.set_source_rgba(0.5, 0.5, 0.5, 0.15);
        cr.set_line_width(thickness);
        let _ = cr.arc(center_x, center_y, radius, 0.0, 2.0 * std::f64::consts::PI);
        let _ = cr.stroke();

        let angle = percent * 2.0 * std::f64::consts::PI;
        let (r, g, b) = if percent < 0.5 {
            (percent * 2.0, 1.0, 0.2)
        } else {
            (1.0, 1.0 - (percent - 0.5) * 2.0, 0.2)
        };
        cr.set_source_rgb(r, g, b);
        
        cr.set_line_width(thickness);
        cr.set_line_cap(gtk4::cairo::LineCap::Round);
        let _ = cr.arc(center_x, center_y, radius, -std::f64::consts::FRAC_PI_2, angle - std::f64::consts::FRAC_PI_2);
        let _ = cr.stroke();

        cr.set_source_rgb(0.9, 0.9, 0.9);
        cr.select_font_face("Sans", gtk4::cairo::FontSlant::Normal, gtk4::cairo::FontWeight::Bold);
        cr.set_font_size(20.0);
        let text = format!("{:.0}%", percent * 100.0);
        if let Ok(extents) = cr.text_extents(&text) {
            cr.move_to(center_x - extents.width() / 2.0, center_y + extents.height() / 2.0);
            let _ = cr.show_text(&text);
        }
    }
}
