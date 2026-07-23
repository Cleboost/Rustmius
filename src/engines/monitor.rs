use crate::config_observer::SshHost;
use crate::engines::ssh::run_remote_command;
use tracing::{debug, instrument};

/// Structure representing various system metrics retrieved from a host.
#[derive(Debug, Default, Clone)]
pub struct SystemMetrics {
    pub os: String,
    pub kernel: String,
    pub uptime: String,
    pub cpu_model: String,
    pub cpu_cores: String,
    pub arch: String,
    pub hostname: String,
    pub ips: Vec<String>,
    pub ram_used: String,
    pub ram_total: String,
    pub ram_percent: f64,
    pub disk_used: String,
    pub disk_total: String,
    pub disk_percent: f64,
    pub cpu_percent: f64,
}

/// Fetches a comprehensive set of system metrics (OS, RAM, Disk, CPU, etc.) from the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias))]
pub async fn fetch_system_metrics(
    host: &SshHost,
    password: Option<&str>,
) -> anyhow::Result<SystemMetrics> {
    debug!("Fetching system metrics for {}", host.alias);
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

    let output = run_remote_command(host, password, cmd).await?;
    let mut metrics = SystemMetrics::default();
    let mut current_section = "";

    for line in output.lines() {
        if line.starts_with("---") && line.ends_with("---") {
            current_section = line;
            continue;
        }
        match current_section {
            "---OS---" => metrics.os = line.to_string(),
            "---KERNEL---" => metrics.kernel = line.to_string(),
            "---UPTIME---" => metrics.uptime = line.to_string(),
            "---CPU_MODEL---" => metrics.cpu_model = line.to_string(),
            "---CPU_CORES---" => metrics.cpu_cores = line.to_string(),
            "---ARCH---" => metrics.arch = line.to_string(),
            "---HOSTNAME---" => metrics.hostname = line.to_string(),
            "---IPS---"
                if !line.is_empty() => {
                    metrics.ips.push(line.to_string());
                }
            "---RAM---" => {
                let parts: Vec<&str> = line.split(" / ").collect();
                if parts.len() == 2 {
                    metrics.ram_used = parts[0].to_string();
                    metrics.ram_total = parts[1].to_string();
                }
            }
            "---RAM_P---" => {
                metrics.ram_percent = line.trim().replace(',', ".").parse().unwrap_or(0.0)
            }
            "---DISK_ALL---" => {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() == 3 {
                    metrics.disk_percent = parts[0]
                        .trim_end_matches('%')
                        .replace(',', ".")
                        .parse::<f64>()
                        .unwrap_or(0.0)
                        / 100.0;
                    metrics.disk_used = parts[1].to_string();
                    metrics.disk_total = parts[2].to_string();
                }
            }
            "---CPU_P---" => {
                metrics.cpu_percent =
                    line.trim().replace(',', ".").parse::<f64>().unwrap_or(0.0) / 100.0
            }
            _ => {}
        }
    }

    Ok(metrics)
}
