use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::{Dimensions, Scroll};
use alacritty_terminal::index::{Column, Line, Point, Side};
use alacritty_terminal::selection::{Selection, SelectionType};
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term, TermMode, test::TermSize};
use alacritty_terminal::vte::ansi::Processor;

/// A detected clickable link in the visible terminal viewport.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DetectedLink {
    /// Normalized URL to open (e.g. `www.example.com` -> `https://www.example.com`).
    pub url: String,
    /// Display segments on the visible screen: `(row, start_col, end_col_inclusive)`.
    pub segments: Vec<(usize, usize, usize)>,
}

pub struct TerminalState {
    term: Term<VoidListener>,
    processor: Processor,
    cols: usize,
    rows: usize,
}

impl TerminalState {
    pub fn new(cols: usize, rows: usize) -> Self {
        let size = TermSize::new(cols.max(1), rows.max(1));
        let config = Config::default();
        let term = Term::new(config, &size, VoidListener);
        Self {
            term,
            processor: Processor::new(),
            cols: cols.max(1),
            rows: rows.max(1),
        }
    }

    pub fn feed_bytes(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.term, bytes);
    }

    pub fn resize(&mut self, cols: usize, rows: usize) -> bool {
        let cols = cols.max(1);
        let rows = rows.max(1);
        if cols == self.cols && rows == self.rows {
            return false;
        }
        self.cols = cols;
        self.rows = rows;
        let size = TermSize::new(cols, rows);
        self.term.resize(size);
        true
    }

    pub fn scroll_display(&mut self, lines: i32) {
        self.term.scroll_display(Scroll::Delta(lines));
    }

    /// Returns `(display_offset, history_size, screen_lines)`.
    pub fn scroll_info(&self) -> (usize, usize, usize) {
        let grid = self.term.grid();
        (grid.display_offset(), grid.history_size(), self.rows)
    }

    pub fn scroll_to_offset(&mut self, target_offset: usize) {
        let grid = self.term.grid();
        let current = grid.display_offset() as i32;
        let target = target_offset.min(grid.history_size()) as i32;
        let delta = target - current;
        if delta != 0 {
            self.term.scroll_display(Scroll::Delta(delta));
        }
    }

    pub fn term(&self) -> &Term<VoidListener> {
        &self.term
    }

    fn screen_to_point(&self, col: usize, row: usize) -> Point {
        let display_offset = self.term.grid().display_offset() as i32;
        Point::new(
            Line(row as i32 - display_offset),
            Column(col.min(self.cols.saturating_sub(1))),
        )
    }

    pub fn start_selection(&mut self, col: usize, row: usize) {
        let point = self.screen_to_point(col, row);
        self.term.selection = Some(Selection::new(SelectionType::Simple, point, Side::Left));
    }

    pub fn update_selection(&mut self, col: usize, row: usize) {
        let point = self.screen_to_point(col, row);
        if let Some(ref mut sel) = self.term.selection {
            sel.update(point, Side::Right);
        }
    }

    pub fn clear_selection(&mut self) {
        self.term.selection = None;
    }

    pub fn selection_text(&self) -> Option<String> {
        self.term.selection_to_string().filter(|s| !s.is_empty())
    }

    pub fn is_bracketed_paste(&self) -> bool {
        self.term.mode().contains(TermMode::BRACKETED_PASTE)
    }

    /// Detects a link (OSC 8 hyperlink or plain-text URL, including wrapped lines)
    /// at the given visible screen cell `(col, row)`.
    pub fn link_at(&self, col: usize, row: usize) -> Option<DetectedLink> {
        if row >= self.rows || col >= self.cols {
            return None;
        }

        let grid = self.term.grid();
        let display_offset = grid.display_offset() as i32;
        let target_line = Line(row as i32 - display_offset);
        let cell = &grid[target_line][Column(col)];

        // 1. Explicit OSC 8 terminal hyperlink
        if let Some(hl) = cell.hyperlink() {
            let target_uri = hl.uri().to_string();
            let target_id = hl.id().to_string();
            let mut segments = Vec::new();
            for r in 0..self.rows {
                let l = Line(r as i32 - display_offset);
                let mut seg_start: Option<usize> = None;
                for c in 0..self.cols {
                    let matches = grid[l][Column(c)]
                        .hyperlink()
                        .is_some_and(|h| h.id() == target_id && h.uri() == target_uri);
                    if matches {
                        if seg_start.is_none() {
                            seg_start = Some(c);
                        }
                    } else if let Some(s) = seg_start.take() {
                        segments.push((r, s, c - 1));
                    }
                }
                if let Some(s) = seg_start {
                    segments.push((r, s, self.cols - 1));
                }
            }
            if !segments.is_empty() && !target_uri.is_empty() {
                return Some(DetectedLink {
                    url: target_uri,
                    segments,
                });
            }
        }

        // 2. Plain-text URL across wrapped logical lines
        let last_col = Column(self.cols - 1);
        let mut start_row = row;
        while start_row > 0 {
            let prev_line = Line((start_row - 1) as i32 - display_offset);
            if grid[prev_line][last_col].flags.contains(Flags::WRAPLINE) {
                start_row -= 1;
            } else {
                break;
            }
        }
        let mut end_row = row;
        while end_row + 1 < self.rows {
            let cur_line = Line(end_row as i32 - display_offset);
            if grid[cur_line][last_col].flags.contains(Flags::WRAPLINE) {
                end_row += 1;
            } else {
                break;
            }
        }

        let mut text = String::with_capacity((end_row - start_row + 1) * self.cols);
        let mut cell_map: Vec<(usize, usize)> =
            Vec::with_capacity((end_row - start_row + 1) * self.cols);
        let mut target_char_idx: Option<usize> = None;

        for r in start_row..=end_row {
            let l = Line(r as i32 - display_offset);
            for c in 0..self.cols {
                let cell = &grid[l][Column(c)];
                if cell.flags.contains(Flags::WIDE_CHAR_SPACER) {
                    if r == row && c == col && !cell_map.is_empty() {
                        target_char_idx = Some(cell_map.len() - 1);
                    }
                    continue;
                }
                if r == row && c == col {
                    target_char_idx = Some(cell_map.len());
                }
                let ch = if cell.c == '\0' { ' ' } else { cell.c };
                text.push(ch);
                cell_map.push((r, c));
            }
        }

        let target_idx = target_char_idx?;
        for (char_start, char_end, url) in find_urls_in_line(&text) {
            if target_idx >= char_start && target_idx < char_end {
                let mut segments: Vec<(usize, usize, usize)> = Vec::new();
                for &(r, c) in &cell_map[char_start..char_end] {
                    if let Some(last) = segments.last_mut()
                        && last.0 == r
                        && last.2 + 1 == c
                    {
                        last.2 = c;
                    } else {
                        segments.push((r, c, c));
                    }
                }
                return Some(DetectedLink { url, segments });
            }
        }

        None
    }
}

const URL_SCHEMES: &[&str] = &[
    "https://", "http://", "ftp://", "ftps://", "ssh://", "ws://", "wss://", "file://",
];

fn is_url_char(c: char) -> bool {
    c.is_ascii()
        && !c.is_ascii_control()
        && !c.is_ascii_whitespace()
        && !matches!(c, '<' | '>' | '"' | '`' | '{' | '}' | '|' | '\\' | '^')
}

fn is_www_boundary(prev: Option<char>) -> bool {
    match prev {
        None => true,
        Some(c) => {
            c.is_whitespace()
                || matches!(
                    c,
                    '(' | '[' | '<' | '\'' | '"' | '`' | '{' | ',' | ';' | '=' | ':'
                )
        }
    }
}

/// Finds URLs in a logical line of text, returning `(char_start, char_end_exclusive, normalized_url)`.
fn find_urls_in_line(line: &str) -> Vec<(usize, usize, String)> {
    let chars: Vec<char> = line.chars().collect();
    let len = chars.len();
    let mut results = Vec::new();
    let mut i = 0;

    while i < len {
        // Check scheme prefix (case-insensitive)
        let mut matched_scheme_len: Option<usize> = None;
        let mut prepend_https = false;

        for &scheme in URL_SCHEMES {
            let slen = scheme.len();
            if i + slen <= len
                && chars[i..i + slen]
                    .iter()
                    .zip(scheme.chars())
                    .all(|(a, b)| a.to_ascii_lowercase() == b)
            {
                matched_scheme_len = Some(slen);
                break;
            }
        }

        if matched_scheme_len.is_none()
            && i + 4 <= len
            && is_www_boundary(i.checked_sub(1).map(|p| chars[p]))
            && chars[i..i + 4]
                .iter()
                .zip("www.".chars())
                .all(|(a, b)| a.to_ascii_lowercase() == b)
        {
            matched_scheme_len = Some(4);
            prepend_https = true;
        }

        let Some(prefix_len) = matched_scheme_len else {
            i += 1;
            continue;
        };

        let mut end = i + prefix_len;
        while end < len && is_url_char(chars[end]) {
            end += 1;
        }

        // Trim trailing punctuation while preserving balanced parentheses/brackets
        while end > i + prefix_len {
            let last = chars[end - 1];
            let trim = match last {
                '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"' => true,
                ')' => {
                    let opens = chars[i..end].iter().filter(|&&c| c == '(').count();
                    let closes = chars[i..end].iter().filter(|&&c| c == ')').count();
                    closes > opens
                }
                ']' => {
                    let opens = chars[i..end].iter().filter(|&&c| c == '[').count();
                    let closes = chars[i..end].iter().filter(|&&c| c == ']').count();
                    closes > opens
                }
                _ => false,
            };
            if trim {
                end -= 1;
            } else {
                break;
            }
        }

        let body_len = end.saturating_sub(i + prefix_len);
        let valid = if prepend_https {
            // Require at least `www.a.b`
            let body: String = chars[i + prefix_len..end].iter().collect();
            let host = body.split('/').next().unwrap_or("");
            host.contains('.') && !host.starts_with('.') && !host.ends_with('.')
        } else {
            body_len > 0 && chars[i + prefix_len] != '.'
        };

        if valid {
            let raw: String = chars[i..end].iter().collect();
            let url = if prepend_https {
                format!("https://{raw}")
            } else {
                raw
            };
            results.push((i, end, url));
            i = end;
        } else {
            i += 1;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feeds_ansi_text() {
        let mut state = TerminalState::new(80, 24);
        state.feed_bytes(b"Hello \x1b[32mRustmius\x1b[0m!");
        let content = state.term().renderable_content();
        let mut line = String::new();
        for cell in content.display_iter {
            if cell.point.line.0 == 0 && cell.point.column.0 < 15 {
                line.push(cell.c);
            }
        }
        assert!(line.starts_with("Hello Rustmius!"));
    }

    #[test]
    fn detects_plain_and_punctuated_urls() {
        let mut state = TerminalState::new(80, 10);
        state.feed_bytes(b"Visit (https://github.com/Cleboost/Rustmius), or www.rust-lang.org.");

        let gh = state.link_at(10, 0).expect("should detect github url");
        assert_eq!(gh.url, "https://github.com/Cleboost/Rustmius");
        assert_eq!(gh.segments, vec![(0, 7, 42)]);

        // Hovering outside the URL should return None
        assert!(state.link_at(0, 0).is_none());
        assert!(state.link_at(43, 0).is_none()); // ')'

        let rust = state.link_at(55, 0).expect("should detect www url");
        assert_eq!(rust.url, "https://www.rust-lang.org");
        assert_eq!(rust.segments, vec![(0, 49, 65)]);
    }

    #[test]
    fn preserves_balanced_parentheses_in_urls() {
        let urls =
            find_urls_in_line("See https://en.wikipedia.org/wiki/Rust_(programming_language).");
        assert_eq!(urls.len(), 1);
        assert_eq!(
            urls[0].2,
            "https://en.wikipedia.org/wiki/Rust_(programming_language)"
        );
    }

    #[test]
    fn detects_wrapped_urls_across_rows() {
        let mut state = TerminalState::new(20, 5);
        state.feed_bytes(b"go https://example.com/very/long/path/here ok");

        // Link starts on row 0 col 3 and wraps onto row 1 and row 2
        let link_row0 = state.link_at(5, 0).expect("detect on row 0");
        let link_row1 = state.link_at(5, 1).expect("detect on row 1");
        assert_eq!(link_row0.url, "https://example.com/very/long/path/here");
        assert_eq!(link_row0, link_row1);
        assert!(link_row0.segments.len() >= 2);
    }

    #[test]
    fn detects_osc8_hyperlinks() {
        let mut state = TerminalState::new(80, 5);
        state.feed_bytes(b"Click \x1b]8;;https://rustmius.dev\x1b\\here\x1b]8;;\x1b\\ now");

        let link = state.link_at(7, 0).expect("should detect OSC 8 link");
        assert_eq!(link.url, "https://rustmius.dev");
        assert_eq!(link.segments, vec![(0, 6, 9)]);
        assert!(state.link_at(0, 0).is_none());
    }
}
