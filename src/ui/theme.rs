use gtk4::gdk::RGBA;
use std::str::FromStr;
use vte4::prelude::*;

/// A terminal color scheme: default foreground/background/cursor colors plus the
/// 16-entry ANSI palette (8 normal + 8 bright).
pub struct TerminalTheme {
    pub name: &'static str,
    pub foreground: &'static str,
    pub background: &'static str,
    pub cursor: &'static str,
    pub palette: [&'static str; 16],
}

/// Built-in terminal themes. The first entry is used as the fallback default.
pub static THEMES: &[TerminalTheme] = &[
    TerminalTheme {
        name: "Dracula",
        foreground: "#f8f8f2",
        background: "#282a36",
        cursor: "#f8f8f2",
        palette: [
            "#21222c", "#ff5555", "#50fa7b", "#f1fa8c", "#bd93f9", "#ff79c6", "#8be9fd", "#f8f8f2",
            "#6272a4", "#ff6e6e", "#69ff94", "#ffffa5", "#d6acff", "#ff92df", "#a4ffff", "#ffffff",
        ],
    },
    TerminalTheme {
        name: "Nord",
        foreground: "#d8dee9",
        background: "#2e3440",
        cursor: "#d8dee9",
        palette: [
            "#3b4252", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#88c0d0", "#e5e9f0",
            "#4c566a", "#bf616a", "#a3be8c", "#ebcb8b", "#81a1c1", "#b48ead", "#8fbcbb", "#eceff4",
        ],
    },
    TerminalTheme {
        name: "Tokyo Night",
        foreground: "#c0caf5",
        background: "#1a1b26",
        cursor: "#c0caf5",
        palette: [
            "#15161e", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#a9b1d6",
            "#414868", "#f7768e", "#9ece6a", "#e0af68", "#7aa2f7", "#bb9af7", "#7dcfff", "#c0caf5",
        ],
    },
    TerminalTheme {
        name: "Catppuccin Mocha",
        foreground: "#cdd6f4",
        background: "#1e1e2e",
        cursor: "#f5e0dc",
        palette: [
            "#45475a", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5", "#bac2de",
            "#585b70", "#f38ba8", "#a6e3a1", "#f9e2af", "#89b4fa", "#f5c2e7", "#94e2d5", "#a6adc8",
        ],
    },
    TerminalTheme {
        name: "Gruvbox Dark",
        foreground: "#ebdbb2",
        background: "#282828",
        cursor: "#ebdbb2",
        palette: [
            "#282828", "#cc241d", "#98971a", "#d79921", "#458588", "#b16286", "#689d6a", "#a89984",
            "#928374", "#fb4934", "#b8bb26", "#fabd2f", "#83a598", "#d3869b", "#8ec07c", "#ebdbb2",
        ],
    },
    TerminalTheme {
        name: "One Dark",
        foreground: "#abb2bf",
        background: "#282c34",
        cursor: "#528bff",
        palette: [
            "#282c34", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2", "#abb2bf",
            "#5c6370", "#e06c75", "#98c379", "#e5c07b", "#61afef", "#c678dd", "#56b6c2", "#ffffff",
        ],
    },
    TerminalTheme {
        name: "Solarized Dark",
        foreground: "#839496",
        background: "#002b36",
        cursor: "#93a1a1",
        palette: [
            "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#eee8d5",
            "#002b36", "#cb4b16", "#586e75", "#657b83", "#839496", "#6c71c4", "#93a1a1", "#fdf6e3",
        ],
    },
    TerminalTheme {
        name: "Monokai",
        foreground: "#f8f8f2",
        background: "#272822",
        cursor: "#f8f8f0",
        palette: [
            "#272822", "#f92672", "#a6e22e", "#f4bf75", "#66d9ef", "#ae81ff", "#a1efe4", "#f8f8f2",
            "#75715e", "#f92672", "#a6e22e", "#f4bf75", "#66d9ef", "#ae81ff", "#a1efe4", "#f9f8f5",
        ],
    },
    TerminalTheme {
        name: "Solarized Light",
        foreground: "#657b83",
        background: "#fdf6e3",
        cursor: "#586e75",
        palette: [
            "#073642", "#dc322f", "#859900", "#b58900", "#268bd2", "#d33682", "#2aa198", "#eee8d5",
            "#002b36", "#cb4b16", "#586e75", "#657b83", "#839496", "#6c71c4", "#93a1a1", "#fdf6e3",
        ],
    },
    TerminalTheme {
        name: "GitHub Light",
        foreground: "#24292e",
        background: "#ffffff",
        cursor: "#24292e",
        palette: [
            "#24292e", "#d73a49", "#28a745", "#dbab09", "#0366d6", "#5a32a3", "#0598bc", "#6a737d",
            "#959da5", "#cb2431", "#22863a", "#b08800", "#005cc5", "#5a32a3", "#3192aa", "#d1d5da",
        ],
    },
];

/// The default theme name, used when the config value is empty or unrecognized.
pub fn default_theme_name() -> &'static str {
    THEMES[0].name
}

/// Returns the theme matching `name`, falling back to the default if not found.
pub fn get_theme(name: &str) -> &'static TerminalTheme {
    THEMES.iter().find(|t| t.name == name).unwrap_or(&THEMES[0])
}

/// The names of all built-in themes, in display order.
pub fn theme_names() -> Vec<&'static str> {
    THEMES.iter().map(|t| t.name).collect()
}

fn parse_color(hex: &str, fallback: RGBA) -> RGBA {
    RGBA::from_str(hex).unwrap_or(fallback)
}

/// Applies the given theme's colors to a single terminal.
pub fn apply_to_terminal(theme: &TerminalTheme, terminal: &vte4::Terminal) {
    let fg = parse_color(theme.foreground, RGBA::WHITE);
    let bg = parse_color(theme.background, RGBA::BLACK);
    let cursor = parse_color(theme.cursor, fg);
    let palette: Vec<RGBA> = theme
        .palette
        .iter()
        .map(|c| parse_color(c, RGBA::BLACK))
        .collect();
    let palette_refs: Vec<&RGBA> = palette.iter().collect();
    terminal.set_colors(Some(&fg), Some(&bg), &palette_refs);
    terminal.set_color_cursor(Some(&cursor));
}

/// Re-applies a theme to every open terminal currently living in the notebook.
/// Session pages are vertical boxes whose direct children include the VTE
/// terminal; other page types (explorer, monitor, docker) are skipped.
pub fn apply_to_open_terminals(notebook: &gtk4::Notebook, theme: &TerminalTheme) {
    for i in 0..notebook.n_pages() {
        let Some(page) = notebook.nth_page(Some(i)) else { continue };
        let Some(bx) = page.downcast_ref::<gtk4::Box>() else { continue };
        let mut child = bx.first_child();
        while let Some(c) = child {
            if let Some(term) = c.downcast_ref::<vte4::Terminal>() {
                apply_to_terminal(theme, term);
            }
            child = c.next_sibling();
        }
    }
}
