# Rustmius: High-Performance, Local-First SSH Client

## 1. Vision & Goals
Rustmius is a 100% local, privacy-first, and high-performance alternative to Termius.
- **Privacy**: No cloud, no external sync. Everything is stored locally.
- **Performance**: Native Rust + GTK4 + VTE for a lightweight, snappy experience.
- **UX**: Keyboard-first navigation via a HUD (Heads-Up Display) and intelligent tiling.

---

## 2. Phased Roadmap

### 🟢 MVP 0.8: "Ship Ultra Vite"
**Goal**: Replace the standard CLI SSH workflow with a superior UI experience.
- **Config Sync**: Watch and import `~/.ssh/config` using the `notify` crate.
- **The HUD (`Ctrl+K`)**: Instant fuzzy-search for host aliases and IP addresses.
- **Terminal Engine**: Native `vte4-rs` implementation.
- **SSH Connectivity**: Use `libssh2-rs` for stability. Support `ssh-agent` and password prompts.
- **Simplified Layout**: Tabs or simple horizontal splits (no full BSP yet).
- **Secret Management**: Native `libsecret` integration with a clean fallback.

### 🔥 MVP 1.0: "The Rustmius Moment"
**Goal**: Deliver the power-user "claque" with advanced automation.
- **BSP Tiling**: Full Binary Space Partitioning engine for automatic layout management.
- **Intelligent Layouts**: Layout policies (Master-Stack, Grid) that adapt to user preferences.
- **Pure Keyboard Navigation**: Leader-key driven workflow for tile management and navigation.
- **Advanced State**: Saved Workspaces and session recovery.
- **Engine Evolution**: Explore transition to `russh` (pure Rust).

---

## 3. Technical Stack
- **Language**: Rust (Edition 2024).
- **UI Framework**: GTK4 (No `libadwaita` for a neutral, custom-styled look).
- **Terminal**: `vte4-rs`.
- **SSH Engine**: `libssh2-rs` (MVP) -> `russh` (Long-term).
- **Data Persistence**: SQLite for metadata (tags, preferences) + `notify` for config watching.
- **Secret Storage**: `libsecret` (GNOME/KDE Keyrings).

---

## 4. Architecture Detail: The "Hybrid Observer"
- **Discovery**: Rustmius treats `~/.ssh/config` as the source of truth for host data.
- **Augmentation**: A local SQLite database layers "Rustmius metadata" (tags, custom colors, layout states) on top of the system config.
- **Reactive State**: The Rust core manages sessions and layouts asynchronously, ensuring the GTK main loop never freezes.

---

## 5. UI/UX Design Principles
- **Sober Aesthetic**: Clean borders, minimal padding, and professional color schemes via GTK CSS.
- **HUD-First**: Primary interaction happens through `Ctrl+K` to find and connect.
- **Automaticity**: Tiles are placed automatically based on the active layout policy (e.g., "Split the widest tile").
- **Zero Distraction**: No scrollbars, tabs, or buttons unless absolutely necessary for the current context.
