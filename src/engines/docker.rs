use crate::config_observer::SshHost;
use crate::engines::ssh::run_remote_command;
use tracing::{info, debug, instrument};

/// Retrieves Docker statistics (containers, images, running, paused) from the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias))]
pub async fn get_docker_stats(host: &SshHost, password: Option<&str>) -> anyhow::Result<Vec<String>> {
    debug!("Fetching Docker stats for {}", host.alias);
    let cmd = "DOCKER_BIN=$(if [ -w /var/run/docker.sock ]; then echo 'docker'; else echo 'sudo -n docker'; fi); \
               OUT=$($DOCKER_BIN info --format '{{.Containers}} {{.Images}} {{.ContainersRunning}} {{.ContainersPaused}}' 2>/dev/null); \
               if [ -z \"$OUT\" ]; then \
                 echo $($DOCKER_BIN ps -aq 2>/dev/null | wc -l) $($DOCKER_BIN images -q 2>/dev/null | wc -l) $($DOCKER_BIN ps -q 2>/dev/null | wc -l) $($DOCKER_BIN ps -f status=paused -q 2>/dev/null | wc -l); \
               else \
                 echo $OUT; \
               fi";
    
    let output = run_remote_command(host, password, cmd).await?;
    let trimmed = output.trim();
    let last_line = trimmed.lines().last().unwrap_or("");
    Ok(last_line.split_whitespace().map(|s| s.to_string()).collect())
}

/// Lists Docker items (containers or images) from the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, is_containers = is_containers))]
pub async fn list_docker_items(host: &SshHost, password: Option<&str>, is_containers: bool) -> anyhow::Result<String> {
    debug!("Listing Docker {} for {}", if is_containers { "containers" } else { "images" }, host.alias);
    let cmd = if is_containers {
        "DOCKER_BIN=$(if [ -w /var/run/docker.sock ]; then echo 'docker'; else echo 'sudo -n docker'; fi); $DOCKER_BIN ps -a --format '{{.Names}}\t{{.Status}}\t{{.Image}}'"
    } else {
        "DOCKER_BIN=$(if [ -w /var/run/docker.sock ]; then echo 'docker'; else echo 'sudo -n docker'; fi); $DOCKER_BIN images --format '{{.Repository}}\t{{.Tag}}\t{{.Size}}'"
    };
    run_remote_command(host, password, cmd).await
}

/// Performs a Docker action (e.g., start, stop, rm, rmi) on a specific item.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, action = %action, item = %item_name))]
pub async fn perform_docker_action(host: &SshHost, password: Option<&str>, action: &str, item_name: &str) -> anyhow::Result<()> {
    info!("Performing Docker action '{}' on '{}' at {}", action, item_name, host.alias);
    let safe_name = item_name.replace('\'', "'\\''");
    let cmd = format!("DOCKER_BIN=$(if [ -w /var/run/docker.sock ]; then echo 'docker'; else echo 'sudo -n docker'; fi); $DOCKER_BIN {} '{}'", action, safe_name);
    let _ = run_remote_command(host, password, &cmd).await?;
    Ok(())
}
