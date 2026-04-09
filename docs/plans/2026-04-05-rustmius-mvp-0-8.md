# Rustmius MVP 0.8 Implementation Plan

> **For Gemini:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** A native Linux SSH client with a keyboard-driven HUD for connecting to hosts from `~/.ssh/config`.

**Architecture:** Async Rust core managing SSH sessions via `libssh2`, integrated with a GTK4 UI using `vte4` for terminal emulation. A background watcher syncs system SSH configs into an in-memory search index.

**Tech Stack:** Rust (2024), GTK4, `vte4-rs`, `ssh2` (libssh2 bindings), `tokio`, `notify`, `nucleo-matcher` (fuzzy search).

---

### Task 1: Project Scaffolding & Dependencies

**Files:**
- Modify: `Cargo.toml`

**Step 1: Update Cargo.toml with core dependencies**

```toml
[package]
name = "rustmius"
version = "0.8.0"
edition = "2024"

[dependencies]
gtk4 = "0.9"
vte4 = "0.9"
ssh2 = "0.9"
tokio = { version = "1", features = ["full"] }
notify = "6.1"
nucleo-matcher = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
directories = "5.0"
futures = "0.3"
```

**Step 2: Run cargo check to verify dependencies**

Run: `cargo check`
Expected: Success (downloads and compiles metadata).

**Step 3: Commit**

```bash
git add Cargo.toml
git commit -m "chore: scaffold project with MVP 0.8 dependencies"
```

---

### Task 2: SSH Config Observer

**Files:**
- Create: `src/config_observer.rs`
- Modify: `src/main.rs`

**Step 1: Write the failing test for config parsing**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_ssh_config_simple() {
        let config = "Host my-server\n  HostName 1.2.3.4\n  User root";
        let hosts = parse_ssh_config(config);
        assert_eq!(hosts[0].alias, "my-server");
        assert_eq!(hosts[0].hostname, "1.2.3.4");
    }
}
```

**Step 2: Implement the parsing logic and watcher**

```rust
// src/config_observer.rs
pub struct SshHost {
    pub alias: String,
    pub hostname: String,
    pub user: Option<String>,
}

pub fn parse_ssh_config(content: &str) -> Vec<SshHost> {
    // Basic parser for ~/.ssh/config
    let mut hosts = Vec::new();
    // ... implementation logic ...
    hosts
}
```

**Step 3: Run tests**

Run: `cargo test config_observer`
Expected: PASS

**Step 4: Commit**

```bash
git add src/config_observer.rs
git commit -m "feat: add ssh config observer and parser"
```

---

### Task 3: The HUD (Fuzzy Search UI)

**Files:**
- Create: `src/ui/hud.rs`

**Step 1: Create the GtkPopover for the HUD**

```rust
// src/ui/hud.rs
use gtk4::prelude::*;

pub fn create_hud_popover() -> gtk4::Popover {
    let popover = gtk4::Popover::new();
    let entry = gtk4::Entry::new();
    entry.set_placeholder_text(Some("Search hosts..."));
    popover.set_child(Some(&entry));
    popover
}
```

**Step 2: Integrate nucleo-matcher for fuzzy results**

**Step 3: Commit**

```bash
git add src/ui/hud.rs
git commit -m "feat: implement HUD search overlay"
```

---

### Task 4: Basic Terminal UI (GTK4 + VTE)

**Files:**
- Modify: `src/main.rs`
- Create: `src/ui/window.rs`

**Step 1: Setup the main GTK window with VTE**

```rust
// src/ui/window.rs
use gtk4::prelude::*;
use vte4::prelude::*;

pub fn build_ui(app: &gtk4::Application) {
    let window = gtk4::ApplicationWindow::builder()
        .application(app)
        .title("Rustmius")
        .default_width(800)
        .default_height(600)
        .build();

    let terminal = vte4::Terminal::new();
    window.set_child(Some(&terminal));
    window.present();
}
```

**Step 2: Commit**

```bash
git add src/ui/window.rs src/main.rs
git commit -m "feat: basic GTK4 window with VTE terminal"
```

---

### Task 5: SSH Connectivity (libssh2)

**Files:**
- Create: `src/ssh_engine.rs`

**Step 1: Implement the SSH session handshake**

```rust
// src/ssh_engine.rs
use ssh2::Session;
use std::net::TcpStream;

pub fn connect(host: &str, user: &str) -> anyhow::Result<Session> {
    let tcp = TcpStream::connect(format!("{}:22", host))?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;
    Ok(sess)
}
```

**Step 2: Commit**

```bash
git add src/ssh_engine.rs
git commit -m "feat: implement core SSH connection engine"
```
