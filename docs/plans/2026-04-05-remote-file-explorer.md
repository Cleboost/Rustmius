# Remote File Explorer (SFTP) Implementation Plan

> **For Gemini:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Provide a native file explorer for remote servers via SFTP, accessible from a toolbar above each SSH terminal.

**Architecture:** 
- **UI (Toolbar)**: Wrap the terminal in a `GtkBox` with a top toolbar containing a "Folder" button.
- **UI (Explorer)**: Create a new component `FileExplorer` using a `GtkListBox` to display remote files.
- **Backend (SFTP)**: Use the `ssh2` crate to open an SFTP session when the explorer tab is requested.
- **Data Flow**: Use async Rust to fetch file lists and update the UI without freezing.

**Tech Stack:** Rust, GTK4, `ssh2` (SFTP), `tokio` (async).

---

### Task 1: Terminal Toolbar & Tab Navigation

**Files:**
- Modify: `src/ui/window.rs`

**Step 1: Wrap terminal in a Box with a Toolbar and Folder button**

```rust
// src/ui/window.rs
// Inside the action_logic for ServerAction::Connect:
let terminal_container = gtk4::Box::new(gtk4::Orientation::Vertical, 0);
let toolbar = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
toolbar.set_margin_top(4);
toolbar.set_margin_bottom(4);
toolbar.set_margin_start(6);

let explorer_btn = gtk4::Button::from_icon_name("folder-remote-symbolic");
explorer_btn.add_css_class("flat");
toolbar.append(&explorer_btn);

terminal_container.append(&toolbar);
terminal_container.append(&terminal);
```

**Step 2: Implement the "Open Explorer Tab" logic when clicking the Folder button**

**Step 3: Commit**

```bash
git add src/ui/window.rs
git commit -m "ui: add session toolbar with explorer button and tab navigation"
```

---

### Task 2: SFTP Backend Engine

**Files:**
- Create: `src/sftp_engine.rs`
- Modify: `src/main.rs`

**Step 1: Implement basic SFTP connection and listing logic**

**Step 2: Commit**

```bash
git add src/sftp_engine.rs src/main.rs
git commit -m "feat: implement SFTP backend for remote file listing"
```

---

### Task 3: File Explorer UI Component

**Files:**
- Create: `src/ui/file_explorer.rs`
- Modify: `src/ui/mod.rs`

**Step 1: Build the basic File Explorer view with a list**

**Step 2: Commit**

```bash
git add src/ui/file_explorer.rs src/ui/mod.rs
git commit -m "ui: implement remote file explorer view"
```
