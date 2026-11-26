use crate::core::ssh::SshCommand;
use crate::core::terminal::TerminalState;
use crate::ui::theme;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::vte::ansi::{Color as AnsiColor, NamedColor};
use egui::{Color32, Event, FontId, Key, PointerButton, Pos2, Rect, Sense, Ui, Vec2};
use tokio::sync::mpsc::UnboundedSender;

fn ansi_to_color32(color: AnsiColor, is_fg: bool) -> Color32 {
    match color {
        AnsiColor::Spec(rgb) => Color32::from_rgb(rgb.r, rgb.g, rgb.b),
        AnsiColor::Named(named) => match named {
            NamedColor::Black => Color32::from_rgb(0x14, 0x17, 0x1F),
            NamedColor::Red => Color32::from_rgb(0xEF, 0x44, 0x44),
            NamedColor::Green => Color32::from_rgb(0x00, 0xC4, 0x8C),
            NamedColor::Yellow => Color32::from_rgb(0xF5, 0x9E, 0x0B),
            NamedColor::Blue => Color32::from_rgb(0x3B, 0x82, 0xF6),
            NamedColor::Magenta => Color32::from_rgb(0xA8, 0x55, 0xF7),
            NamedColor::Cyan => Color32::from_rgb(0x06, 0xB6, 0xD4),
            NamedColor::White => Color32::from_rgb(0xE6, 0xEA, 0xF2),
            NamedColor::BrightBlack => Color32::from_rgb(0x64, 0x74, 0x8B),
            NamedColor::BrightRed => Color32::from_rgb(0xF8, 0x71, 0x71),
            NamedColor::BrightGreen => Color32::from_rgb(0x34, 0xD3, 0x99),
            NamedColor::BrightYellow => Color32::from_rgb(0xFB, 0xBF, 0x24),
            NamedColor::BrightBlue => Color32::from_rgb(0x60, 0xA5, 0xFA),
            NamedColor::BrightMagenta => Color32::from_rgb(0xC0, 0x84, 0xFC),
            NamedColor::BrightCyan => Color32::from_rgb(0x22, 0xD3, 0xEE),
            NamedColor::BrightWhite => Color32::WHITE,
            NamedColor::Foreground => theme::TERM_FG,
            NamedColor::Background => theme::TERM_BG,
            _ => {
                if is_fg {
                    theme::TERM_FG
                } else {
                    theme::TERM_BG
                }
            }
        },
        AnsiColor::Indexed(idx) => {
            if idx < 16 {
                let named = match idx {
                    0 => NamedColor::Black,
                    1 => NamedColor::Red,
                    2 => NamedColor::Green,
                    3 => NamedColor::Yellow,
                    4 => NamedColor::Blue,
                    5 => NamedColor::Magenta,
                    6 => NamedColor::Cyan,
                    7 => NamedColor::White,
                    8 => NamedColor::BrightBlack,
                    9 => NamedColor::BrightRed,
                    10 => NamedColor::BrightGreen,
                    11 => NamedColor::BrightYellow,
                    12 => NamedColor::BrightBlue,
                    13 => NamedColor::BrightMagenta,
                    14 => NamedColor::BrightCyan,
                    _ => NamedColor::BrightWhite,
                };
                ansi_to_color32(AnsiColor::Named(named), is_fg)
            } else if idx < 232 {
                let i = idx - 16;
                let r = (i / 36) * 51;
                let g = ((i % 36) / 6) * 51;
                let b = (i % 6) * 51;
                Color32::from_rgb(r, g, b)
            } else {
                let gray = 8 + (idx - 232) * 10;
                Color32::from_rgb(gray, gray, gray)
            }
        }
    }
}

fn brighten_ansi(color: AnsiColor) -> AnsiColor {
    match color {
        AnsiColor::Named(NamedColor::Black) | AnsiColor::Indexed(0) => {
            AnsiColor::Named(NamedColor::BrightBlack)
        }
        AnsiColor::Named(NamedColor::Red) | AnsiColor::Indexed(1) => {
            AnsiColor::Named(NamedColor::BrightRed)
        }
        AnsiColor::Named(NamedColor::Green) | AnsiColor::Indexed(2) => {
            AnsiColor::Named(NamedColor::BrightGreen)
        }
        AnsiColor::Named(NamedColor::Yellow) | AnsiColor::Indexed(3) => {
            AnsiColor::Named(NamedColor::BrightYellow)
        }
        AnsiColor::Named(NamedColor::Blue) | AnsiColor::Indexed(4) => {
            AnsiColor::Named(NamedColor::BrightBlue)
        }
        AnsiColor::Named(NamedColor::Magenta) | AnsiColor::Indexed(5) => {
            AnsiColor::Named(NamedColor::BrightMagenta)
        }
        AnsiColor::Named(NamedColor::Cyan) | AnsiColor::Indexed(6) => {
            AnsiColor::Named(NamedColor::BrightCyan)
        }
        AnsiColor::Named(NamedColor::White) | AnsiColor::Indexed(7) => {
            AnsiColor::Named(NamedColor::BrightWhite)
        }
        other => other,
    }
}

pub fn render_terminal(
    ui: &mut Ui,
    terminal: &mut TerminalState,
    cmd_tx: &UnboundedSender<SshCommand>,
    font_size: f32,
    copy_on_select: bool,
    bright_bold: bool,
) {
    let font_id = FontId::monospace(font_size);
    let (cell_w, cell_h) = ui.fonts(|f| {
        let w = f.glyph_width(&font_id, 'W');
        let h = f.row_height(&font_id);
        (w.max(6.0), h.max(12.0))
    });

    const PAD: f32 = 12.0;
    let avail = ui.available_size();
    let inner_w = (avail.x - PAD * 2.0).max(cell_w * 10.0);
    let inner_h = (avail.y - PAD * 2.0).max(cell_h * 4.0);
    let cols = ((inner_w / cell_w).floor() as usize).max(10);
    let rows = ((inner_h / cell_h).floor() as usize).max(4);

    if terminal.resize(cols, rows) {
        let _ = cmd_tx.send(SshCommand::Resize {
            cols: cols as u32,
            rows: rows as u32,
        });
    }

    let (rect, response) = ui.allocate_exact_size(avail, Sense::click_and_drag());
    if response.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
    }
    let init_id = response.id.with("terminal_initial_focus");
    let did_focus = ui.data_mut(|d| {
        let already = d.get_temp::<bool>(init_id).unwrap_or(false);
        if !already {
            d.insert_temp(init_id, true);
        }
        already
    });
    if !did_focus || response.clicked() {
        response.request_focus();
    }

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, theme::TERM_BG);

    let origin = Pos2::new(rect.min.x + PAD, rect.min.y + PAD);

    // Mouse selection
    let pos_to_cell = |pos: Pos2| -> (usize, usize) {
        let col =
            (((pos.x - origin.x) / cell_w).floor().max(0.0) as usize).min(cols.saturating_sub(1));
        let row =
            (((pos.y - origin.y) / cell_h).floor().max(0.0) as usize).min(rows.saturating_sub(1));
        (col, row)
    };
    if response.drag_started_by(PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            let (col, row) = pos_to_cell(pos);
            terminal.start_selection(col, row);
        }
    } else if response.dragged_by(PointerButton::Primary) {
        if let Some(pos) = response.interact_pointer_pos() {
            let (col, row) = pos_to_cell(pos);
            terminal.update_selection(col, row);
        }
    } else if response.drag_stopped_by(PointerButton::Primary) {
        if copy_on_select && let Some(selected) = terminal.selection_text() {
            ui.ctx().copy_text(selected);
        }
    } else if response.clicked() {
        terminal.clear_selection();
    }

    // Cell rendering
    let content = terminal.term().renderable_content();
    let display_offset = content.display_offset as i32;
    let cursor_point = content.cursor.point;
    let selection = content.selection;

    for indexed in content.display_iter {
        let row_idx = indexed.point.line.0 + display_offset;
        if row_idx < 0 || row_idx >= rows as i32 {
            continue;
        }
        let col_idx = indexed.point.column.0;
        if col_idx >= cols {
            continue;
        }

        let x = origin.x + col_idx as f32 * cell_w;
        let y = origin.y + row_idx as f32 * cell_h;
        let cell_rect = Rect::from_min_size(Pos2::new(x, y), Vec2::new(cell_w, cell_h));

        let fg_ansi = if bright_bold && indexed.flags.contains(Flags::BOLD) {
            brighten_ansi(indexed.fg)
        } else {
            indexed.fg
        };
        let mut fg = ansi_to_color32(fg_ansi, true);
        let mut bg = ansi_to_color32(indexed.bg, false);
        if indexed.flags.contains(Flags::INVERSE) {
            std::mem::swap(&mut fg, &mut bg);
        }

        let is_selected = selection
            .map(|s| s.contains_cell(&indexed, cursor_point, content.cursor.shape))
            .unwrap_or(false);
        if is_selected {
            painter.rect_filled(cell_rect, 0.0, theme::ACCENT_BLUE.linear_multiply(0.4));
        } else if bg != theme::TERM_BG {
            painter.rect_filled(cell_rect, 0.0, bg);
        }

        if ('\u{2800}'..='\u{28FF}').contains(&indexed.c) {
            paint_braille(&painter, cell_rect, indexed.c, fg);
        } else if indexed.c != ' ' && indexed.c != '\0' {
            let mut buf = [0u8; 4];
            let s = indexed.c.encode_utf8(&mut buf);
            painter.text(
                Pos2::new(x, y),
                egui::Align2::LEFT_TOP,
                s,
                font_id.clone(),
                fg,
            );
        }
    }

    // Cursor
    let cur_row = cursor_point.line.0 + display_offset;
    if cur_row >= 0 && cur_row < rows as i32 && cursor_point.column.0 < cols {
        let cx = origin.x + cursor_point.column.0 as f32 * cell_w;
        let cy = origin.y + cur_row as f32 * cell_h;
        let cur_rect = Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(cell_w, cell_h));
        painter.rect_filled(cur_rect, 0.0, theme::ACCENT_GREEN.linear_multiply(0.55));
    }

    // Keyboard & scroll input
    if response.has_focus() {
        let events = ui.input(|i| i.events.clone());
        for ev in events {
            match ev {
                Event::Copy => {
                    if let Some(selected) = terminal.selection_text() {
                        ui.ctx().copy_text(selected);
                        terminal.clear_selection();
                    } else {
                        let _ = cmd_tx.send(SshCommand::Data(vec![0x03]));
                    }
                }
                Event::Paste(text) => {
                    let bytes = if terminal.is_bracketed_paste() {
                        let sanitized = text.replace("\x1b[200~", "").replace("\x1b[201~", "");
                        format!("\x1b[200~{sanitized}\x1b[201~").into_bytes()
                    } else {
                        text.into_bytes()
                    };
                    let _ = cmd_tx.send(SshCommand::Data(bytes));
                }
                Event::Text(text) => {
                    let _ = cmd_tx.send(SshCommand::Data(text.into_bytes()));
                }
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => {
                    if modifiers.ctrl
                        && let Some(byte) = ctrl_key_byte(key)
                    {
                        let _ = cmd_tx.send(SshCommand::Data(vec![byte]));
                        continue;
                    }
                    if let Some(seq) = special_key_bytes(key) {
                        let _ = cmd_tx.send(SshCommand::Data(seq.to_vec()));
                    }
                }
                Event::MouseWheel { delta, .. } => {
                    let lines = (delta.y / 20.0).round() as i32;
                    if lines != 0 {
                        terminal.scroll_display(lines);
                    }
                }
                _ => {}
            }
        }
    }
}

fn paint_braille(painter: &egui::Painter, cell: Rect, ch: char, fg: Color32) {
    let bits = (ch as u32) - 0x2800;
    if bits == 0 {
        return;
    }
    const DOTS: [(u32, f32, f32); 8] = [
        (0x01, 0.28, 0.14),
        (0x02, 0.28, 0.38),
        (0x04, 0.28, 0.62),
        (0x40, 0.28, 0.86),
        (0x08, 0.72, 0.14),
        (0x10, 0.72, 0.38),
        (0x20, 0.72, 0.62),
        (0x80, 0.72, 0.86),
    ];
    let r = (cell.width() * 0.14).clamp(1.0, 2.2);
    for (mask, fx, fy) in DOTS {
        if bits & mask != 0 {
            let p = Pos2::new(
                cell.min.x + cell.width() * fx,
                cell.min.y + cell.height() * fy,
            );
            painter.circle_filled(p, r, fg);
        }
    }
}

fn ctrl_key_byte(key: Key) -> Option<u8> {
    let b = match key {
        Key::A => 0x01,
        Key::B => 0x02,
        Key::C => 0x03,
        Key::D => 0x04,
        Key::E => 0x05,
        Key::F => 0x06,
        Key::G => 0x07,
        Key::H => 0x08,
        Key::K => 0x0B,
        Key::L => 0x0C,
        Key::N => 0x0E,
        Key::P => 0x10,
        Key::R => 0x12,
        Key::U => 0x15,
        Key::W => 0x17,
        Key::Z => 0x1A,
        _ => return None,
    };
    Some(b)
}

fn special_key_bytes(key: Key) -> Option<&'static [u8]> {
    let seq: &'static [u8] = match key {
        Key::Enter => b"\r",
        Key::Backspace => b"\x7f",
        Key::Tab => b"\t",
        Key::Escape => b"\x1b",
        Key::ArrowUp => b"\x1b[A",
        Key::ArrowDown => b"\x1b[B",
        Key::ArrowRight => b"\x1b[C",
        Key::ArrowLeft => b"\x1b[D",
        Key::Home => b"\x1b[H",
        Key::End => b"\x1b[F",
        Key::PageUp => b"\x1b[5~",
        Key::PageDown => b"\x1b[6~",
        Key::Delete => b"\x1b[3~",
        _ => return None,
    };
    Some(seq)
}
