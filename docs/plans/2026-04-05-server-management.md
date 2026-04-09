# Server Management (Edit & Delete) Implementation Plan

> **For Gemini:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Enable users to modify or remove existing SSH connections from Rustmius and `~/.ssh/config`.

**Architecture:** 
- **Config Layer**: Add `delete_host_from_config` in `src/config_observer.rs`.
- **UI Layer**: 
    - Update `ServerList` cards to include Edit/Delete buttons (native GTK icons).
    - Refactor `add_server_dialog` to support pre-filled fields for editing.
- **Security**: Ensure passwords are also removed from the Keyring when a server is deleted.

---

### Task 1: Config & Keyring Management (The Engine)

**Files:**
- Modify: `src/config_observer.rs`

**Step 1: Implement deletion and update logic**

```rust
// src/config_observer.rs
pub fn delete_host_from_config(alias: &str) -> anyhow::Result<()> {
    let path = get_default_config_path().ok_or_else(|| anyhow::anyhow!("No config path"))?;
    if !path.exists() { return Ok(()); }
    
    let content = std::fs::read_to_string(&path)?;
    let mut new_lines = Vec::new();
    let mut skip = false;
    let target_alias = alias.to_lowercase();

    for line in content.lines() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("host ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() > 1 && parts[1] == target_alias {
                skip = true;
                continue;
            } else {
                skip = false;
            }
        }
        
        if skip {
            continue;
        }
        
        new_lines.push(line);
    }
    
    std::fs::write(path, new_lines.join("\n"))?;
    Ok(())
}
```

**Step 2: Commit**

```bash
git add src/config_observer.rs
git commit -m "feat: implement ssh config deletion logic"
```

---

### Task 2: Refactor Dialog & ServerList UI

**Files:**
- Modify: `src/ui/add_server_dialog.rs`
- Modify: `src/ui/server_list.rs`

**Step 1: Update show_add_server_dialog to accept initial data**

```rust
// src/ui/add_server_dialog.rs
pub fn show_server_dialog<F>(parent: &gtk4::Window, initial_host: Option<&SshHost>, on_save: F)
where F: Fn(SshHost, String) + 'static
{
    // ... pre-fill entries if initial_host is Some ...
}
```

**Step 2: Add Edit/Delete buttons to ServerList cards**

```rust
// src/ui/server_list.rs
pub enum ServerAction {
    Connect(SshHost),
    Edit(SshHost),
    Delete(SshHost),
}
// Update add_host_row to include buttons and trigger these actions
```

**Step 3: Commit**

```bash
git add src/ui/add_server_dialog.rs src/ui/server_list.rs
git commit -m "ui: add edit/delete buttons and refactor dialog for edition"
```

---

### Task 3: Final Wiring in Window

**Files:**
- Modify: `src/ui/window.rs`

**Step 1: Handle Edit and Delete actions in build_ui**
- For Delete: Call `delete_host_from_config` + Clear Keyring + Refresh list.
- For Edit: Call `delete_host_from_config` + `show_server_dialog` + `add_host_to_config` + Refresh.

**Step 2: Commit**

```bash
git add src/ui/window.rs
git commit -m "feat: functional server edition and deletion with keyring cleanup"
```
