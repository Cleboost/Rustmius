# Rustmius

A high-performance, privacy-first SSH client for Linux. Built with gtk-rs (GTK4).

## Features

- **Keyboard-driven HUD** (`Ctrl+K`) — fuzzy-search and connect to hosts instantly
- **SSH config sync** — automatically watches and imports `~/.ssh/config`
- **Integrated terminals** — embedded VTE terminal widgets inside the GTK4 GUI
- **Secret management** — integrates with GNOME/KDE keyrings via `libsecret`
- **Server management** — add, edit, and delete hosts directly from the UI
- **No cloud, no telemetry** — everything stays local

## Tech Stack

| Component | Technology |
|-----------|------------|
| Language | Rust 2024 Edition |
| UI | GTK4 (no libadwaita) |
| Terminal | vte4-rs |
| SSH | libssh2-rs |
| Async | tokio |
| Fuzzy search | nucleo-matcher |
| Config watching | notify |
| Secret storage | oo7 (libsecret) |

## Prerequisites

- Rust (Edition 2024 / recent toolchain)
- GTK4 development libraries
- VTE GTK4 development libraries
- libssh2 development libraries
- libsecret development libraries

On Debian/Ubuntu:

```bash
sudo apt install libgtk-4-dev libvte-2.91-gtk4-dev libssh2-1-dev libsecret-1-dev
```

On Arch Linux:

```bash
sudo pacman -S gtk4 vte3 libssh2 libsecret
```

On Fedora:

```bash
sudo dnf install gtk4-devel vte291-devel libssh2-devel libsecret-devel
```

## Build & Run

```bash
cargo build --release
cargo run --release
```

## Usage

1. **Open the HUD** — press `Ctrl+K` to search your hosts
2. **Connect** — select a host from the fuzzy search results
3. **Add a server** — use the server list UI to add new hosts to `~/.ssh/config`
4. **Manage servers** — edit or delete existing entries from the server list

## Architecture

Rustmius is a complete GTK4 desktop application (built with gtk-rs). It treats `~/.ssh/config` as the source of truth for host data. A background watcher (`notify`) detects changes and rebuilds the in-memory search index. The GUI provides a HUD overlay for quick host discovery and embedded VTE terminal widgets for active SSH sessions.

```
src/
├── main.rs              # Entry point, GTK4 app setup, askpass handler
├── config_observer.rs   # SSH config parser and file watcher
├── ssh_engine.rs        # SSH session management (libssh2)
└── ui/
    ├── mod.rs
    ├── window.rs        # Main application window
    ├── hud.rs           # Ctrl+K fuzzy search overlay
    ├── server_list.rs   # Host list management
    └── add_server_dialog.rs  # Add new server dialog
```

## Roadmap

### Current (v0.8)
- SSH config sync with file watching
- HUD fuzzy search
- VTE terminal integration
- Password & ssh-agent authentication
- Server list CRUD

### Planned (v1.0)
- BSP tiling for terminal panels
- Saved workspaces and session recovery
- Pure Rust SSH engine (`russh`)
- Advanced keyboard navigation with leader keys

## License

MIT
