use alacritty_terminal::event::VoidListener;
use alacritty_terminal::grid::Scroll;
use alacritty_terminal::term::{Config, Term, test::TermSize};
use alacritty_terminal::vte::ansi::Processor;

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

    pub fn cols(&self) -> usize {
        self.cols
    }

    pub fn rows(&self) -> usize {
        self.rows
    }

    pub fn term(&self) -> &Term<VoidListener> {
        &self.term
    }
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
}
