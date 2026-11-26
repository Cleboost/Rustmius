# 🦀 Rustmius v3

[![Release](https://img.shields.io/github/v/release/Cleboost/Rustmius?style=flat-square&color=orange)](https://github.com/Cleboost/Rustmius/releases)
[![License](https://img.shields.io/github/license/Cleboost/Rustmius?style=flat-square)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-linux-lightgrey?style=flat-square)](https://github.com/Cleboost/Rustmius)

**Rustmius v3** is a fast, native, open-source SSH client and host manager inspired by Termius, rewritten from the ground up in pure Rust using `egui`/`eframe`, `russh`, and `alacritty_terminal`.

---

## ✨ Features

- **Native Pure-Rust SSH (`russh`)**: Supports SSH Agent (`SSH_AUTH_SOCK` & `~/.bitwarden-ssh-agent.sock`), private keys, password auth, and automatic remote OS detection with vector distro badges.
- **High-Performance Terminal (`alacritty_terminal`)**: Full 256-color/truecolor ANSI rendering, mouse selection, smart `Ctrl+C`/Paste, copy-on-select, bright-bold, customizable font size, and native vector Unicode Braille rendering (for `btop` and TUI graphs).
- **Host Manager & `~/.ssh/config` Discovery**: Grid & list views, sorting, quick-connect bar (`ssh user@host -p 2222`), and one-click import of unimported hosts detected from `~/.ssh/config`.
- **End-to-End Encrypted Sync Vault (GitHub Gist)**: Optionally sync your hosts across machines via a private GitHub Gist (`rustmius-vault.enc`) encrypted locally with **AES-256-GCM + Argon2id**, conflict-free merging with deletion tombstones, and optional auto-unlock on startup.
- **Multilingual UI**: Built-in English and French translations selectable in Settings → General.

> [!NOTE]
> **v3.0.0**: SSH host key verification (`~/.ssh/known_hosts` / TOFU prompt) and SFTP/Keychain/Snippets tabs are planned for upcoming 3.x releases.

---

## 🚀 Building from Source

### NixOS / Nix Flakes (recommended)

```bash
git clone https://github.com/Cleboost/Rustmius.git
cd Rustmius
nix develop path:. --command cargo run --release
```

### Standard Linux (Cargo)

Ensure OpenGL / Wayland / X11 development libraries are installed (`libxkbcommon`, `wayland`, `libGL`, `libx11`, `libxcursor`, `libxrandr`, `libxi`), then:

```bash
cargo build --release
./target/release/rustmius
```

### Running Tests

```bash
cargo test
```

---

## 🏗️ Architecture

- `src/core/`: Async SSH client (`russh` + `tokio`) and terminal emulator state (`alacritty_terminal`).
- `src/storage/`: Local vault & settings persistence (`0600` permissions on Unix), `~/.ssh/config` parser, and E2E encrypted GitHub Gist sync engine.
- `src/ui/`: Custom frameless Termius-inspired `egui` interface with Inter typography and Phosphor icons.
- `src/i18n.rs`: Compile-time checked English & French localization tables.

## 📄 License

Licensed under **AGPL-3.0-or-later**. See [LICENSE](LICENSE).
