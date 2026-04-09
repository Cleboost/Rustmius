# Robust Alias Handling Implementation Plan

> **For Gemini:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Support spaces in server names and prevent duplicate aliases.

**Architecture:** 
- **Parser**: Update `parse_ssh_config` to handle quoted aliases and multi-word aliases correctly.
- **Persistence**: Update `add_host_to_config` to wrap aliases in double quotes if they contain spaces.
- **Validation**: Add a check in `show_server_dialog` to verify if the alias already exists in the current list.

---

### Task 1: Robust Parser & Quoted Persistence

**Files:**
- Modify: `src/config_observer.rs`

**Step 1: Update parser to handle spaces and quotes**

```rust
// src/config_observer.rs
pub fn parse_ssh_config(content: &str) -> Vec<SshHost> {
    let mut hosts = Vec::new();
    let mut current_host: Option<SshHost> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Handle quoted values or multi-word values for Host
        let parts: Vec<&str> = line.splitn(2, |c: char| c.is_whitespace()).collect();
        if parts.len() < 2 {
            continue;
        }

        let key = parts[0].to_lowercase();
        let mut value = parts[1].trim();
        
        // Remove quotes if present
        if value.starts_with('"') && value.ends_with('"') && value.len() > 1 {
            value = &value[1..value.len()-1];
        }

        match key.as_str() {
            "host" => {
                if let Some(host) = current_host.take() {
                    if !host.alias.is_empty() && !host.hostname.is_empty() {
                        hosts.push(host);
                    }
                }
                current_host = Some(SshHost {
                    alias: value.to_string(),
                    hostname: String::new(),
                    user: None,
                });
            }
            "hostname" => {
                if let Some(ref mut host) = current_host {
                    host.hostname = value.to_string();
                }
            }
            "user" => {
                if let Some(ref mut host) = current_host {
                    host.user = Some(value.to_string());
                }
            }
            _ => {}
        }
    }

    if let Some(host) = current_host {
        if !host.alias.is_empty() && !host.hostname.is_empty() {
            hosts.push(host);
        }
    }

    hosts
}

pub fn add_host_to_config(host: &SshHost) -> anyhow::Result<()> {
    let path = get_default_config_path().ok_or_else(|| anyhow::anyhow!("Could not find SSH config path"))?;
    
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut content = if path.exists() {
        std::fs::read_to_string(&path)?
    } else {
        String::new()
    };

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    // Wrap alias in quotes if it has spaces
    let alias_quoted = if host.alias.contains(' ') {
        format!("\"{}\"", host.alias)
    } else {
        host.alias.clone()
    };

    let entry = format!(
        "\nHost {}\n    HostName {}\n    User {}\n",
        alias_quoted,
        host.hostname,
        host.user.as_deref().unwrap_or("root")
    );

    content.push_str(&entry);
    std::fs::write(path, content)?;
    Ok(())
}
```

**Step 2: Commit**

```bash
git add src/config_observer.rs
git commit -m "feat: support spaces in aliases using quotes in ssh config"
```

---

### Task 2: Duplicate Name Validation in UI

**Files:**
- Modify: `src/ui/add_server_dialog.rs`

**Step 1: Update show_server_dialog to accept existing aliases and show an error if duplicate**

```rust
// src/ui/add_server_dialog.rs
pub fn show_server_dialog<F>(
    parent: &gtk4::Window, 
    initial_host: Option<&SshHost>, 
    existing_aliases: Vec<String>,
    on_save: F
) where F: Fn(SshHost, String) + 'static {
    // ... logic to check alias against existing_aliases ...
    // If alias exists and not editing same host, show error label
}
```

**Step 2: Commit**

```bash
git add src/ui/add_server_dialog.rs
git commit -m "ui: prevent duplicate server aliases in the creation dialog"
```

---

### Task 3: Wiring in window.rs

**Files:**
- Modify: `src/ui/window.rs`

**Step 1: Update calls to show_server_dialog to pass existing aliases**

**Step 2: Commit**

```bash
git add src/ui/window.rs
git commit -m "feat: complete robust alias management"
```
