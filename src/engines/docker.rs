use crate::config_observer::SshHost;
use crate::engines::ssh::run_remote_command;
use tracing::{info, instrument};

/// Shell prologue that picks the right Docker invocation (`docker` if the socket
/// is writable, otherwise `sudo -n docker`) and exposes it as `$DOCKER_BIN`.
/// Centralized so the detection logic stays in one place.
const DOCKER_BIN: &str = "DOCKER_BIN=$(if [ -w /var/run/docker.sock ]; then echo 'docker'; else echo 'sudo -n docker'; fi); ";

/// Retrieves Docker statistics (containers, images, running, paused) from the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias))]
pub async fn get_docker_stats(
    host: &SshHost,
    password: Option<&str>,
) -> anyhow::Result<Vec<String>> {
    tracing::trace!("Fetching Docker stats for {}", host.alias);
    // `concat` (not `format!`) so the Go-template `{{...}}` braces stay literal.
    let cmd = [
        DOCKER_BIN,
        "OUT=$($DOCKER_BIN info --format '{{.Containers}} {{.Images}} {{.ContainersRunning}} {{.ContainersPaused}}' 2>/dev/null); \
         if [ -z \"$OUT\" ]; then \
           echo $($DOCKER_BIN ps -aq 2>/dev/null | wc -l) $($DOCKER_BIN images -q 2>/dev/null | wc -l) $($DOCKER_BIN ps -q 2>/dev/null | wc -l) $($DOCKER_BIN ps -f status=paused -q 2>/dev/null | wc -l); \
         else \
           echo $OUT; \
         fi",
    ].concat();

    let output = run_remote_command(host, password, &cmd).await?;
    let last_line = output.trim().lines().last().unwrap_or("");
    Ok(last_line.split_whitespace().map(str::to_owned).collect())
}

/// Lists Docker items (containers or images) from the remote host.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, is_containers = is_containers))]
pub async fn list_docker_items(
    host: &SshHost,
    password: Option<&str>,
    is_containers: bool,
) -> anyhow::Result<String> {
    tracing::trace!(
        "Listing Docker {} for {}",
        if is_containers {
            "containers"
        } else {
            "images"
        },
        host.alias
    );
    let subcmd = if is_containers {
        "$DOCKER_BIN ps -a --format '{{.Names}}\t{{.Status}}\t{{.Image}}'"
    } else {
        "$DOCKER_BIN images --format '{{.Repository}}\t{{.Tag}}\t{{.Size}}'"
    };
    let cmd = [DOCKER_BIN, subcmd].concat();
    run_remote_command(host, password, &cmd).await
}

/// Performs a Docker action (e.g., start, stop, rm, rmi) on a specific item.
#[instrument(skip(password), fields(host = %host.hostname, alias = %host.alias, action = %action, item = %item_name))]
pub async fn perform_docker_action(
    host: &SshHost,
    password: Option<&str>,
    action: &str,
    item_name: &str,
) -> anyhow::Result<()> {
    info!(
        "Performing Docker action '{}' on '{}' at {}",
        action, item_name, host.alias
    );
    let safe_name = item_name.replace('\'', "'\\''");
    let cmd = format!("{DOCKER_BIN}$DOCKER_BIN {action} '{safe_name}'");
    run_remote_command(host, password, &cmd).await?;
    Ok(())
}
