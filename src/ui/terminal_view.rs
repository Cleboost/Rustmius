use crate::core::ssh::SshCommand;
use crate::core::terminal::{DetectedLink, TerminalState};
use crate::i18n::{Language, tr};
use crate::ui::icons::{self, IconStyle, regular};
use crate::ui::theme;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::vte::ansi::{Color as AnsiColor, NamedColor};
use egui::{
    Align2, Color32, CornerRadius, Event, FontId, Key, PointerButton, Pos2, Rect, Sense, Stroke,
    StrokeKind, Ui, Vec2, pos2, vec2,
};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;

const DWELL_DELAY_SECS: f64 = 0.42;
const STILL_TOLERANCE_PX: f32 = 4.0;
const LEAVE_GRACE_SECS: f64 = 0.22;
const COPY_FEEDBACK_SECS: f64 = 1.25;

#[cfg(target_os = "macos")]
const MOD_KEY_LABEL: &str = "⌘";
#[cfg(not(target_os = "macos"))]
const MOD_KEY_LABEL: &str = "Ctrl";

#[derive(Clone, Default)]
struct LinkHoverState {
    active_link: Option<DetectedLink>,
    dwell_anchor_pos: Option<Pos2>,
    dwell_since: Option<f64>,
    popover_open: bool,
    leave_since: Option<f64>,
    popover_rect: Option<Rect>,
    anchor_rect: Option<Rect>,
    last_rendered_link: Option<DetectedLink>,
    last_anchor_rect: Option<Rect>,
    copied_at: Option<f64>,
}

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
    lang: Language,
) {
    let font_id = FontId::monospace(font_size);
    let (cell_w, cell_h) = ui.fonts_mut(|f| {
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
        if !did_focus {
            ui.ctx().request_repaint();
        }
    }

    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, theme::TERM_BG);

    let origin = Pos2::new(rect.min.x + PAD, rect.min.y + PAD);
    let grid_rect = Rect::from_min_size(
        origin,
        Vec2::new(cols as f32 * cell_w, rows as f32 * cell_h),
    );

    let pos_to_cell = |pos: Pos2| -> (usize, usize) {
        let col =
            (((pos.x - origin.x) / cell_w).floor().max(0.0) as usize).min(cols.saturating_sub(1));
        let row =
            (((pos.y - origin.y) / cell_h).floor().max(0.0) as usize).min(rows.saturating_sub(1));
        (col, row)
    };

    // Link hover state
    let link_state_id = response.id.with("terminal_link_hover");
    let mut link_state: LinkHoverState =
        ui.data_mut(|d| d.get_temp(link_state_id).unwrap_or_default());

    let (now, ctrl_down, pointer_pos) = ui.input(|i| {
        (
            i.time,
            i.modifiers.ctrl || i.modifiers.command,
            i.pointer.hover_pos(),
        )
    });

    let over_popover = link_state.popover_open
        && pointer_pos.is_some_and(|p| {
            link_state
                .popover_rect
                .is_some_and(|r| r.expand(4.0).contains(p))
        });

    let hovered_cell_and_link = if !over_popover {
        pointer_pos
            .filter(|p| grid_rect.contains(*p) && response.hovered())
            .and_then(|p| {
                let (col, row) = pos_to_cell(p);
                terminal.link_at(col, row).map(|link| (row, link))
            })
    } else {
        None
    };

    // Cursor icon
    if response.hovered() && !over_popover {
        if hovered_cell_and_link.is_some() && ctrl_down {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        } else {
            ui.ctx().set_cursor_icon(egui::CursorIcon::Text);
        }
    }

    // Mouse selection & Ctrl+Click link opening
    let ctrl_link_hovered = ctrl_down && hovered_cell_and_link.is_some();
    if response.drag_started_by(PointerButton::Primary) {
        if !ctrl_link_hovered
            && let Some(pos) = ui
                .input(|i| i.pointer.press_origin())
                .or_else(|| response.interact_pointer_pos())
        {
            let (col, row) = pos_to_cell(pos);
            terminal.start_selection(col, row);
            if let Some(cur) = response.interact_pointer_pos() {
                let (cur_col, cur_row) = pos_to_cell(cur);
                terminal.update_selection(cur_col, cur_row);
            }
            link_state.popover_open = false;
            link_state.dwell_since = None;
        }
    } else if response.dragged_by(PointerButton::Primary) {
        if !ctrl_link_hovered && let Some(pos) = response.interact_pointer_pos() {
            let (col, row) = pos_to_cell(pos);
            terminal.update_selection(col, row);
            link_state.popover_open = false;
            link_state.dwell_since = None;
        }
    } else if response.drag_stopped_by(PointerButton::Primary) {
        if copy_on_select && let Some(selected) = terminal.selection_text() {
            ui.ctx().copy_text(selected);
        }
    } else if response.clicked_by(PointerButton::Primary) {
        if let Some((_, ref link)) = hovered_cell_and_link
            && ctrl_down
        {
            ui.ctx().open_url(egui::OpenUrl::new_tab(link.url.clone()));
            link_state.popover_open = false;
            link_state.dwell_since = None;
            link_state.leave_since = None;
        } else {
            terminal.clear_selection();
            link_state.popover_open = false;
        }
    } else if response.clicked_by(PointerButton::Secondary) {
        if copy_on_select {
            response.request_focus();
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
        }
        terminal.clear_selection();
        link_state.popover_open = false;
    } else if response.clicked() {
        terminal.clear_selection();
    }

    // Update link hover dwell state machine
    let is_dragging = response.dragged_by(PointerButton::Primary);
    let in_bridge = link_state.popover_open
        && pointer_pos.is_some_and(
            |p| match (link_state.anchor_rect, link_state.popover_rect) {
                (Some(a), Some(pr)) => a.union(pr).expand(4.0).contains(p),
                _ => false,
            },
        );

    if is_dragging {
        link_state.active_link = None;
        link_state.popover_open = false;
        link_state.dwell_since = None;
        link_state.dwell_anchor_pos = None;
        link_state.leave_since = None;
    } else if over_popover {
        link_state.leave_since = None;
    } else if let Some((hovered_row, link)) = hovered_cell_and_link {
        let same_link = link_state
            .active_link
            .as_ref()
            .is_some_and(|prev| prev == &link);
        if !same_link {
            link_state.active_link = Some(link);
            link_state.dwell_anchor_pos = pointer_pos;
            link_state.dwell_since = Some(now);
            link_state.popover_open = false;
            link_state.leave_since = None;
            link_state.anchor_rect = None;
            link_state.copied_at = None;
            ui.ctx()
                .request_repaint_after(Duration::from_secs_f64(DWELL_DELAY_SECS));
        } else {
            link_state.leave_since = None;
            if !link_state.popover_open {
                let moved = match (link_state.dwell_anchor_pos, pointer_pos) {
                    (Some(a), Some(b)) => a.distance(b) > STILL_TOLERANCE_PX,
                    _ => true,
                };
                if moved {
                    link_state.dwell_anchor_pos = pointer_pos;
                    link_state.dwell_since = Some(now);
                    ui.ctx()
                        .request_repaint_after(Duration::from_secs_f64(DWELL_DELAY_SECS));
                } else {
                    let since = *link_state.dwell_since.get_or_insert(now);
                    let elapsed = now - since;
                    if elapsed >= DWELL_DELAY_SECS {
                        link_state.popover_open = true;
                        link_state.anchor_rect =
                            segment_rect_for_row(&link, hovered_row, origin, cell_w, cell_h);
                    } else {
                        ui.ctx().request_repaint_after(Duration::from_secs_f64(
                            (DWELL_DELAY_SECS - elapsed).max(0.016),
                        ));
                    }
                }
            }
        }
    } else if in_bridge {
        link_state.leave_since = None;
    } else if link_state.popover_open {
        let left_at = *link_state.leave_since.get_or_insert(now);
        let elapsed = now - left_at;
        if elapsed >= LEAVE_GRACE_SECS {
            link_state.popover_open = false;
            link_state.active_link = None;
            link_state.leave_since = None;
            link_state.dwell_since = None;
            link_state.dwell_anchor_pos = None;
        } else {
            ui.ctx().request_repaint_after(Duration::from_secs_f64(
                (LEAVE_GRACE_SECS - elapsed).max(0.016),
            ));
        }
    } else {
        link_state.active_link = None;
        link_state.dwell_since = None;
        link_state.dwell_anchor_pos = None;
        link_state.leave_since = None;
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
        } else if ('\u{2500}'..='\u{257F}').contains(&indexed.c)
            && paint_box_drawing(&painter, cell_rect, indexed.c, fg)
        {
            // Rendered as crisp cell-aligned vector primitives.
        } else if ('\u{2580}'..='\u{259F}').contains(&indexed.c)
            && paint_block_element(&painter, cell_rect, indexed.c, fg)
        {
            // Rendered as crisp cell-aligned block primitives.
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

    // Hovered link highlight & underline
    if let Some(ref link) = link_state.active_link {
        let highlight_alpha = if ctrl_down { 0.14 } else { 0.06 };
        let underline_color = if ctrl_down {
            theme::ACCENT_BLUE
        } else {
            Color32::from_rgb(0x60, 0xA5, 0xFA).linear_multiply(0.9)
        };
        let underline_w: f32 = if ctrl_down { 1.5 } else { 1.0 };

        for &(row_idx, start_col, end_col) in &link.segments {
            if row_idx >= rows || start_col >= cols {
                continue;
            }
            let clamped_end = end_col.min(cols - 1);
            let x0 = origin.x + start_col as f32 * cell_w;
            let x1 = origin.x + (clamped_end + 1) as f32 * cell_w;
            let y0 = origin.y + row_idx as f32 * cell_h;
            let y1 = y0 + cell_h;
            let seg_rect = Rect::from_min_max(pos2(x0, y0), pos2(x1, y1));

            painter.rect_filled(
                seg_rect,
                CornerRadius::same(2),
                theme::ACCENT_BLUE.linear_multiply(highlight_alpha),
            );
            painter.line_segment(
                [pos2(x0, y1 - 1.5), pos2(x1, y1 - 1.5)],
                Stroke::new(underline_w, underline_color),
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
    let has_focus = response.has_focus() || !did_focus;
    let is_hovered = response.hovered();
    if has_focus || is_hovered {
        let events = ui.input(|i| i.events.clone());
        for ev in events {
            match ev {
                Event::Copy if has_focus => {
                    let _ = cmd_tx.send(SshCommand::Data(vec![0x03]));
                    link_state.popover_open = false;
                }
                Event::Paste(text) if has_focus => {
                    let bytes = if terminal.is_bracketed_paste() {
                        let sanitized = text.replace("\x1b[200~", "").replace("\x1b[201~", "");
                        format!("\x1b[200~{sanitized}\x1b[201~").into_bytes()
                    } else {
                        text.into_bytes()
                    };
                    let _ = cmd_tx.send(SshCommand::Data(bytes));
                    link_state.popover_open = false;
                }
                Event::Text(text) if has_focus => {
                    let _ = cmd_tx.send(SshCommand::Data(text.into_bytes()));
                    link_state.popover_open = false;
                }
                Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } if has_focus => {
                    if modifiers.ctrl && modifiers.shift {
                        match key {
                            Key::C => {
                                if let Some(selected) = terminal.selection_text() {
                                    ui.ctx().copy_text(selected);
                                    terminal.clear_selection();
                                }
                                continue;
                            }
                            Key::V => {
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                                continue;
                            }
                            _ => {}
                        }
                    }
                    if modifiers.ctrl
                        && let Some(byte) = ctrl_key_byte(key)
                    {
                        let _ = cmd_tx.send(SshCommand::Data(vec![byte]));
                        link_state.popover_open = false;
                        continue;
                    }
                    if let Some(seq) = special_key_bytes(key) {
                        let _ = cmd_tx.send(SshCommand::Data(seq.to_vec()));
                        link_state.popover_open = false;
                    }
                }
                Event::MouseWheel { unit, delta, .. } => {
                    let lines = match unit {
                        egui::MouseWheelUnit::Line => (delta.y * 3.0).round() as i32,
                        egui::MouseWheelUnit::Point => (delta.y / cell_h.max(1.0)).round() as i32,
                        egui::MouseWheelUnit::Page => (delta.y * rows as f32).round() as i32,
                    };
                    if lines != 0 {
                        terminal.scroll_display(lines);
                        link_state.popover_open = false;
                        link_state.active_link = None;
                    }
                }
                _ => {}
            }
        }
    }

    // Cache last link & anchor for smooth fade-out animation
    if link_state.popover_open {
        if let Some(ref link) = link_state.active_link {
            link_state.last_rendered_link = Some(link.clone());
        }
        if let Some(anchor) = link_state.anchor_rect {
            link_state.last_anchor_rect = Some(anchor);
        }
    }

    // Animated hover popover
    render_link_popover(
        ui.ctx(),
        response.id,
        rect,
        &mut link_state,
        ctrl_down,
        now,
        lang,
    );

    ui.data_mut(|d| d.insert_temp(link_state_id, link_state));

    // Scrollbar
    let (_display_offset, history_size, screen_lines) = terminal.scroll_info();
    if history_size > 0 {
        let track_margin_v = 4.0;
        let track_h = (rect.height() - track_margin_v * 2.0).max(1.0);
        let hit_rect = Rect::from_min_max(
            Pos2::new(rect.right() - 12.0, rect.top() + track_margin_v),
            Pos2::new(rect.right(), rect.bottom() - track_margin_v),
        );
        let sb_resp = ui.interact(
            hit_rect,
            response.id.with("scrollbar"),
            Sense::click_and_drag(),
        );

        let total_lines = (history_size + screen_lines) as f32;
        let thumb_h = ((screen_lines as f32 / total_lines) * track_h).clamp(20.0, track_h);
        let scrollable_h = (track_h - thumb_h).max(1.0);

        if (sb_resp.dragged() || sb_resp.clicked())
            && let Some(pos) = sb_resp.interact_pointer_pos()
        {
            let frac = ((pos.y - hit_rect.top() - thumb_h * 0.5) / scrollable_h).clamp(0.0, 1.0);
            let target_offset = ((1.0 - frac) * history_size as f32).round() as usize;
            terminal.scroll_to_offset(target_offset);
        }

        let (display_offset, history_size, _) = terminal.scroll_info();
        let frac = if history_size > 0 {
            1.0 - (display_offset as f32 / history_size as f32)
        } else {
            1.0
        };
        let thumb_y = hit_rect.top() + frac * scrollable_h;
        let active = sb_resp.hovered() || sb_resp.dragged();
        let bar_w = if active { 6.0 } else { 4.0 };
        let thumb_rect = Rect::from_min_size(
            Pos2::new(rect.right() - bar_w - 2.0, thumb_y),
            Vec2::new(bar_w, thumb_h),
        );
        let color = if active {
            Color32::from_white_alpha(150)
        } else {
            Color32::from_white_alpha(75)
        };
        painter.rect_filled(thumb_rect, bar_w * 0.5, color);
    }
}

fn segment_rect_for_row(
    link: &DetectedLink,
    preferred_row: usize,
    origin: Pos2,
    cell_w: f32,
    cell_h: f32,
) -> Option<Rect> {
    let &(row, start_col, end_col) = link
        .segments
        .iter()
        .find(|&&(r, _, _)| r == preferred_row)
        .or_else(|| link.segments.first())?;
    let x0 = origin.x + start_col as f32 * cell_w;
    let x1 = origin.x + (end_col + 1) as f32 * cell_w;
    let y0 = origin.y + row as f32 * cell_h;
    Some(Rect::from_min_max(pos2(x0, y0), pos2(x1, y0 + cell_h)))
}

fn truncate_url(url: &str, max_chars: usize) -> String {
    let count = url.chars().count();
    if count <= max_chars {
        return url.to_owned();
    }
    let keep = max_chars.saturating_sub(1);
    let mut out: String = url.chars().take(keep).collect();
    out.push('…');
    out
}

fn render_link_popover(
    ctx: &egui::Context,
    term_id: egui::Id,
    term_rect: Rect,
    state: &mut LinkHoverState,
    ctrl_down: bool,
    now: f64,
    lang: Language,
) {
    let pop_id = term_id.with("link_hover_popover");
    let anim_t = ctx.animate_bool_with_time(pop_id.with("anim"), state.popover_open, 0.13);
    if anim_t <= 0.01 {
        if !state.popover_open {
            state.popover_rect = None;
        }
        return;
    }

    let (Some(link), Some(anchor)) = (state.last_rendered_link.clone(), state.last_anchor_rect)
    else {
        return;
    };

    let s = tr(lang);
    let display_url = truncate_url(&link.url, 48);
    let url_font = theme::medium(12.5);
    let hint_font = theme::regular(11.0);
    let key_font = theme::semibold(10.0);

    let (url_w, hint_w, key_w) = ctx.fonts_mut(|f| {
        let uw = f
            .layout_no_wrap(display_url.clone(), url_font.clone(), Color32::WHITE)
            .size()
            .x;
        let hw = f
            .layout_no_wrap(
                s.term_link_click_open.to_owned(),
                hint_font.clone(),
                Color32::WHITE,
            )
            .size()
            .x;
        let kw = f
            .layout_no_wrap(MOD_KEY_LABEL.to_owned(), key_font.clone(), Color32::WHITE)
            .size()
            .x;
        (uw, hw, kw)
    });

    // Layout: [pad 10] [globe 14] [gap 7] [url] [gap 10] [ext 12] [gap 6] [copy btn 24] [pad 8]
    let top_row_w = 10.0 + 14.0 + 7.0 + url_w + 10.0 + 12.0 + 6.0 + 24.0 + 8.0;
    let bot_row_w = 10.0 + (key_w + 10.0) + 6.0 + 8.0 + 6.0 + hint_w + 12.0;
    let pop_w = top_row_w.max(bot_row_w).clamp(190.0, 440.0);
    let pop_h = 54.0;

    let screen = ctx.content_rect();
    let place_above = anchor.top() - pop_h - 6.0 >= term_rect.top() + 4.0;
    let ideal_x = anchor.left().min(anchor.center().x - pop_w * 0.35);
    let clamped_x = ideal_x.clamp(
        screen.left() + 8.0,
        (screen.right() - pop_w - 8.0).max(screen.left() + 8.0),
    );
    let y_slide = (1.0 - anim_t) * if place_above { 5.0 } else { -5.0 };
    let pop_y = if place_above {
        anchor.top() - pop_h - 6.0 + y_slide
    } else {
        anchor.bottom() + 6.0 + y_slide
    };

    let just_copied = state
        .copied_at
        .is_some_and(|t| now - t < COPY_FEEDBACK_SECS);
    if let Some(t) = state.copied_at {
        let rem = COPY_FEEDBACK_SECS - (now - t);
        if rem > 0.0 {
            ctx.request_repaint_after(Duration::from_secs_f64(rem.max(0.016)));
        }
    }

    let area_resp = egui::Area::new(pop_id)
        .order(egui::Order::Foreground)
        .fixed_pos(pos2(clamped_x, pop_y))
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(pop_w, pop_h), Sense::hover());
            let painter = ui.painter();

            // Soft drop shadow
            painter.rect_filled(
                rect.translate(vec2(0.0, 4.0)),
                CornerRadius::same(9),
                Color32::from_black_alpha((85.0 * anim_t) as u8),
            );
            // Dark slate glass card matching terminal & titlebar
            let bg = Color32::from_rgba_unmultiplied(0x1F, 0x25, 0x32, (248.0 * anim_t) as u8);
            painter.rect_filled(rect, CornerRadius::same(9), bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(9),
                Stroke::new(
                    1.0_f32,
                    Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, (26.0 * anim_t) as u8),
                ),
                StrokeKind::Inside,
            );

            // Top row: clickable link pill + copy button
            let copy_btn_rect = Rect::from_min_size(
                pos2(rect.right() - 8.0 - 24.0, rect.top() + 5.0),
                vec2(24.0, 24.0),
            );
            let link_btn_rect = Rect::from_min_max(
                pos2(rect.left() + 5.0, rect.top() + 4.0),
                pos2(copy_btn_rect.left() - 4.0, rect.top() + 30.0),
            );

            let link_resp = ui
                .interact(link_btn_rect, pop_id.with("open_link"), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            if link_resp.hovered() {
                painter.rect_filled(
                    link_btn_rect,
                    CornerRadius::same(6),
                    theme::ACCENT_BLUE.linear_multiply(0.18 * anim_t),
                );
            }

            let globe_pos = pos2(link_btn_rect.left() + 12.0, link_btn_rect.center().y);
            icons::paint(
                painter,
                globe_pos,
                regular::GLOBE,
                14.0,
                theme::ACCENT_BLUE.gamma_multiply(anim_t),
                IconStyle::Regular,
            );

            let url_color = if link_resp.hovered() {
                Color32::from_rgb(0x60, 0xA5, 0xFA).gamma_multiply(anim_t)
            } else {
                theme::TOPBAR_TEXT.gamma_multiply(anim_t)
            };
            let url_pos = pos2(link_btn_rect.left() + 25.0, link_btn_rect.center().y);
            let url_clip = Rect::from_min_max(
                pos2(link_btn_rect.left() + 24.0, link_btn_rect.top()),
                pos2(link_btn_rect.right() - 20.0, link_btn_rect.bottom()),
            );
            let url_painter = painter.with_clip_rect(url_clip);
            let text_galley = url_painter.text(
                url_pos,
                Align2::LEFT_CENTER,
                &display_url,
                url_font.clone(),
                url_color,
            );
            if link_resp.hovered() {
                let uy = text_galley.bottom() + 0.5;
                url_painter.line_segment(
                    [pos2(text_galley.left(), uy), pos2(text_galley.right(), uy)],
                    Stroke::new(1.0_f32, url_color),
                );
            }

            // Subtle external-link arrow on the right of the link pill
            let ext_color = if link_resp.hovered() {
                theme::ACCENT_BLUE.gamma_multiply(anim_t)
            } else {
                theme::TOPBAR_MUTED.gamma_multiply(0.75 * anim_t)
            };
            icons::paint(
                painter,
                pos2(link_btn_rect.right() - 10.0, link_btn_rect.center().y),
                regular::ARROW_SQUARE_OUT,
                12.5,
                ext_color,
                IconStyle::Regular,
            );

            if link_resp.clicked() {
                ctx.open_url(egui::OpenUrl::new_tab(link.url.clone()));
                state.popover_open = false;
                state.active_link = None;
                state.leave_since = None;
            }

            // Copy button
            let copy_resp = ui
                .interact(copy_btn_rect, pop_id.with("copy_link"), Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .on_hover_text(s.term_link_copy_tooltip);
            if copy_resp.hovered() {
                painter.rect_filled(
                    copy_btn_rect,
                    CornerRadius::same(6),
                    Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, (22.0 * anim_t) as u8),
                );
            }
            let (copy_glyph, copy_color) = if just_copied {
                (regular::CHECK, theme::ACCENT_GREEN.gamma_multiply(anim_t))
            } else if copy_resp.hovered() {
                (regular::COPY, theme::TOPBAR_TEXT.gamma_multiply(anim_t))
            } else {
                (regular::COPY, theme::TOPBAR_MUTED.gamma_multiply(anim_t))
            };
            icons::paint(
                painter,
                copy_btn_rect.center(),
                copy_glyph,
                13.5,
                copy_color,
                IconStyle::Regular,
            );
            if copy_resp.clicked() {
                ctx.copy_text(link.url.clone());
                state.copied_at = Some(now);
            }

            // Bottom hint row: [Ctrl] + Click to open (or "Link copied!" feedback)
            let hint_y = rect.bottom() - 12.0;
            if just_copied {
                painter.text(
                    pos2(rect.left() + 12.0, hint_y),
                    Align2::LEFT_CENTER,
                    s.term_link_copied,
                    theme::medium(11.0),
                    theme::ACCENT_GREEN.gamma_multiply(anim_t),
                );
            } else {
                let badge_w = (key_w + 10.0).max(24.0);
                let badge_rect = Rect::from_min_size(
                    pos2(rect.left() + 10.0, hint_y - 7.5),
                    vec2(badge_w, 15.0),
                );
                let (badge_bg, badge_border, badge_fg) = if ctrl_down {
                    (
                        theme::ACCENT_BLUE.gamma_multiply(anim_t),
                        theme::ACCENT_BLUE.gamma_multiply(anim_t),
                        Color32::WHITE.gamma_multiply(anim_t),
                    )
                } else {
                    (
                        Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, (16.0 * anim_t) as u8),
                        Color32::from_rgba_unmultiplied(0xFF, 0xFF, 0xFF, (34.0 * anim_t) as u8),
                        theme::TOPBAR_TEXT.gamma_multiply(0.9 * anim_t),
                    )
                };
                painter.rect_filled(badge_rect, CornerRadius::same(4), badge_bg);
                painter.rect_stroke(
                    badge_rect,
                    CornerRadius::same(4),
                    Stroke::new(1.0_f32, badge_border),
                    StrokeKind::Inside,
                );
                painter.text(
                    badge_rect.center(),
                    Align2::CENTER_CENTER,
                    MOD_KEY_LABEL,
                    key_font,
                    badge_fg,
                );

                let hint_color = if ctrl_down {
                    theme::TOPBAR_TEXT.gamma_multiply(anim_t)
                } else {
                    theme::TOPBAR_MUTED.gamma_multiply(anim_t)
                };
                painter.text(
                    pos2(badge_rect.right() + 5.0, hint_y),
                    Align2::LEFT_CENTER,
                    format!("+ {}", s.term_link_click_open),
                    hint_font,
                    hint_color,
                );
            }
        });

    state.popover_rect = Some(area_resp.response.rect);
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

#[derive(Clone, Copy, PartialEq, Eq)]
enum LineWeight {
    None,
    Light,
    Heavy,
    Double,
}

fn paint_box_drawing(painter: &egui::Painter, cell: Rect, ch: char, fg: Color32) -> bool {
    use LineWeight::{Double, Heavy, Light, None};

    let l = cell.min.x;
    let r = cell.max.x;
    let t = cell.min.y;
    let b = cell.max.y;
    let cx = cell.center().x;
    let cy = cell.center().y;
    let light_w = (cell.width() * 0.11).clamp(1.0, 2.0);
    let heavy_w = light_w * 2.0;
    let dbl_gap = (cell.width() * 0.16).clamp(1.5, 3.0);

    // Rounded corners
    match ch {
        '╭' | '╮' | '╯' | '╰' => {
            let rx = (r - cx).min(b - cy) * 0.65;
            let pts: Vec<Pos2> = match ch {
                '╭' => vec![
                    pos2(cx, b),
                    pos2(cx, cy + rx * 0.4),
                    pos2(cx + rx * 0.4, cy),
                    pos2(r, cy),
                ],
                '╮' => vec![
                    pos2(l, cy),
                    pos2(cx - rx * 0.4, cy),
                    pos2(cx, cy + rx * 0.4),
                    pos2(cx, b),
                ],
                '╯' => vec![
                    pos2(l, cy),
                    pos2(cx - rx * 0.4, cy),
                    pos2(cx, cy - rx * 0.4),
                    pos2(cx, t),
                ],
                '╰' => vec![
                    pos2(cx, t),
                    pos2(cx, cy - rx * 0.4),
                    pos2(cx + rx * 0.4, cy),
                    pos2(r, cy),
                ],
                _ => unreachable!(),
            };
            painter.add(egui::Shape::line(pts, Stroke::new(light_w, fg)));
            return true;
        }
        '╱' => {
            painter.line_segment([pos2(l, b), pos2(r, t)], Stroke::new(light_w, fg));
            return true;
        }
        '╲' => {
            painter.line_segment([pos2(l, t), pos2(r, b)], Stroke::new(light_w, fg));
            return true;
        }
        '╳' => {
            painter.line_segment([pos2(l, b), pos2(r, t)], Stroke::new(light_w, fg));
            painter.line_segment([pos2(l, t), pos2(r, b)], Stroke::new(light_w, fg));
            return true;
        }
        _ => {}
    }

    // (left, right, up, down)
    let arms: (LineWeight, LineWeight, LineWeight, LineWeight) = match ch {
        '─' | '┄' | '┈' | '╌' => (Light, Light, None, None),
        '━' | '┅' | '┉' | '╍' => (Heavy, Heavy, None, None),
        '│' | '┆' | '┊' | '╎' => (None, None, Light, Light),
        '┃' | '┇' | '┋' | '╏' => (None, None, Heavy, Heavy),
        '┌' => (None, Light, None, Light),
        '┍' => (None, Heavy, None, Light),
        '┎' => (None, Light, None, Heavy),
        '┏' => (None, Heavy, None, Heavy),
        '┐' => (Light, None, None, Light),
        '┑' => (Heavy, None, None, Light),
        '┒' => (Light, None, None, Heavy),
        '┓' => (Heavy, None, None, Heavy),
        '└' => (None, Light, Light, None),
        '┕' => (None, Heavy, Light, None),
        '┖' => (None, Light, Heavy, None),
        '┗' => (None, Heavy, Heavy, None),
        '┘' => (Light, None, Light, None),
        '┙' => (Heavy, None, Light, None),
        '┚' => (Light, None, Heavy, None),
        '┛' => (Heavy, None, Heavy, None),
        '├' => (None, Light, Light, Light),
        '┝' => (None, Heavy, Light, Light),
        '┞' => (None, Light, Heavy, Light),
        '┟' => (None, Light, Light, Heavy),
        '┠' => (None, Light, Heavy, Heavy),
        '┡' => (None, Heavy, Heavy, Light),
        '┢' => (None, Heavy, Light, Heavy),
        '┣' => (None, Heavy, Heavy, Heavy),
        '┤' => (Light, None, Light, Light),
        '┥' => (Heavy, None, Light, Light),
        '┦' => (Light, None, Heavy, Light),
        '┧' => (Light, None, Light, Heavy),
        '┨' => (Light, None, Heavy, Heavy),
        '┩' => (Heavy, None, Heavy, Light),
        '┪' => (Heavy, None, Light, Heavy),
        '┫' => (Heavy, None, Heavy, Heavy),
        '┬' => (Light, Light, None, Light),
        '┭' => (Heavy, Light, None, Light),
        '┮' => (Light, Heavy, None, Light),
        '┯' => (Heavy, Heavy, None, Light),
        '┰' => (Light, Light, None, Heavy),
        '┱' => (Heavy, Light, None, Heavy),
        '┲' => (Light, Heavy, None, Heavy),
        '┳' => (Heavy, Heavy, None, Heavy),
        '┴' => (Light, Light, Light, None),
        '┵' => (Heavy, Light, Light, None),
        '┶' => (Light, Heavy, Light, None),
        '┷' => (Heavy, Heavy, Light, None),
        '┸' => (Light, Light, Heavy, None),
        '┹' => (Heavy, Light, Heavy, None),
        '┺' => (Light, Heavy, Heavy, None),
        '┻' => (Heavy, Heavy, Heavy, None),
        '┼' => (Light, Light, Light, Light),
        '┽' => (Heavy, Light, Light, Light),
        '┾' => (Light, Heavy, Light, Light),
        '┿' => (Heavy, Heavy, Light, Light),
        '╀' => (Light, Light, Heavy, Light),
        '╁' => (Light, Light, Light, Heavy),
        '╂' => (Light, Light, Heavy, Heavy),
        '╃' => (Heavy, Light, Heavy, Light),
        '╄' => (Light, Heavy, Heavy, Light),
        '╅' => (Heavy, Light, Light, Heavy),
        '╆' => (Light, Heavy, Light, Heavy),
        '╇' => (Heavy, Heavy, Heavy, Light),
        '╈' => (Heavy, Heavy, Light, Heavy),
        '╉' => (Heavy, Light, Heavy, Heavy),
        '╊' => (Light, Heavy, Heavy, Heavy),
        '╋' => (Heavy, Heavy, Heavy, Heavy),
        '═' => (Double, Double, None, None),
        '║' => (None, None, Double, Double),
        '╒' => (None, Double, None, Light),
        '╓' => (None, Light, None, Double),
        '╔' => (None, Double, None, Double),
        '╕' => (Double, None, None, Light),
        '╖' => (Light, None, None, Double),
        '╗' => (Double, None, None, Double),
        '╘' => (None, Double, Light, None),
        '╙' => (None, Light, Double, None),
        '╚' => (None, Double, Double, None),
        '╛' => (Double, None, Light, None),
        '╜' => (Light, None, Double, None),
        '╝' => (Double, None, Double, None),
        '╞' => (None, Double, Light, Light),
        '╟' => (None, Light, Double, Double),
        '╠' => (None, Double, Double, Double),
        '╡' => (Double, None, Light, Light),
        '╢' => (Light, None, Double, Double),
        '╣' => (Double, None, Double, Double),
        '╤' => (Double, Double, None, Light),
        '╥' => (Light, Light, None, Double),
        '╦' => (Double, Double, None, Double),
        '╧' => (Double, Double, Light, None),
        '╨' => (Light, Light, Double, None),
        '╩' => (Double, Double, Double, None),
        '╪' => (Double, Double, Light, Light),
        '╫' => (Light, Light, Double, Double),
        '╬' => (Double, Double, Double, Double),
        '╴' => (Light, None, None, None),
        '╵' => (None, None, Light, None),
        '╶' => (None, Light, None, None),
        '╷' => (None, None, None, Light),
        '╸' => (Heavy, None, None, None),
        '╹' => (None, None, Heavy, None),
        '╺' => (None, Heavy, None, None),
        '╻' => (None, None, None, Heavy),
        '╼' => (Light, Heavy, None, None),
        '╽' => (None, None, Light, Heavy),
        '╾' => (Heavy, Light, None, None),
        '╿' => (None, None, Heavy, Light),
        _ => return false,
    };

    let draw_h = |x0: f32, x1: f32, w: LineWeight| match w {
        None => {}
        Light => {
            painter.line_segment([pos2(x0, cy), pos2(x1, cy)], Stroke::new(light_w, fg));
        }
        Heavy => {
            painter.line_segment([pos2(x0, cy), pos2(x1, cy)], Stroke::new(heavy_w, fg));
        }
        Double => {
            painter.line_segment(
                [pos2(x0, cy - dbl_gap), pos2(x1, cy - dbl_gap)],
                Stroke::new(light_w, fg),
            );
            painter.line_segment(
                [pos2(x0, cy + dbl_gap), pos2(x1, cy + dbl_gap)],
                Stroke::new(light_w, fg),
            );
        }
    };
    let draw_v = |y0: f32, y1: f32, w: LineWeight| match w {
        None => {}
        Light => {
            painter.line_segment([pos2(cx, y0), pos2(cx, y1)], Stroke::new(light_w, fg));
        }
        Heavy => {
            painter.line_segment([pos2(cx, y0), pos2(cx, y1)], Stroke::new(heavy_w, fg));
        }
        Double => {
            painter.line_segment(
                [pos2(cx - dbl_gap, y0), pos2(cx - dbl_gap, y1)],
                Stroke::new(light_w, fg),
            );
            painter.line_segment(
                [pos2(cx + dbl_gap, y0), pos2(cx + dbl_gap, y1)],
                Stroke::new(light_w, fg),
            );
        }
    };

    let (wl, wr, wu, wd) = arms;
    draw_h(l, cx, wl);
    draw_h(cx, r, wr);
    draw_v(t, cy, wu);
    draw_v(cy, b, wd);
    true
}

fn paint_block_element(painter: &egui::Painter, cell: Rect, ch: char, fg: Color32) -> bool {
    let l = cell.min.x;
    let r = cell.max.x;
    let t = cell.min.y;
    let b = cell.max.y;
    let w = cell.width();
    let h = cell.height();
    let cx = cell.center().x;
    let cy = cell.center().y;

    let fill = |rect: Rect, color: Color32| {
        painter.rect_filled(rect, 0.0, color);
    };
    match ch {
        '▀' => fill(Rect::from_min_max(pos2(l, t), pos2(r, cy)), fg),
        '▁'..='█' => {
            let eighths = (ch as u32 - '▁' as u32 + 1) as f32 / 8.0;
            fill(Rect::from_min_max(pos2(l, b - h * eighths), pos2(r, b)), fg);
        }
        '▉'..='▏' => {
            let eighths = (8 - (ch as u32 - '▉' as u32 + 1)) as f32 / 8.0;
            fill(Rect::from_min_max(pos2(l, t), pos2(l + w * eighths, b)), fg);
        }
        '▐' => fill(Rect::from_min_max(pos2(cx, t), pos2(r, b)), fg),
        '░' => fill(cell, fg.linear_multiply(0.25)),
        '▒' => fill(cell, fg.linear_multiply(0.50)),
        '▓' => fill(cell, fg.linear_multiply(0.75)),
        '▔' => fill(Rect::from_min_max(pos2(l, t), pos2(r, t + h / 8.0)), fg),
        '▕' => fill(Rect::from_min_max(pos2(r - w / 8.0, t), pos2(r, b)), fg),
        '▖' => fill(Rect::from_min_max(pos2(l, cy), pos2(cx, b)), fg),
        '▗' => fill(Rect::from_min_max(pos2(cx, cy), pos2(r, b)), fg),
        '▘' => fill(Rect::from_min_max(pos2(l, t), pos2(cx, cy)), fg),
        '▙' => {
            fill(Rect::from_min_max(pos2(l, t), pos2(cx, b)), fg);
            fill(Rect::from_min_max(pos2(cx, cy), pos2(r, b)), fg);
        }
        '▚' => {
            fill(Rect::from_min_max(pos2(l, t), pos2(cx, cy)), fg);
            fill(Rect::from_min_max(pos2(cx, cy), pos2(r, b)), fg);
        }
        '▛' => {
            fill(Rect::from_min_max(pos2(l, t), pos2(r, cy)), fg);
            fill(Rect::from_min_max(pos2(l, cy), pos2(cx, b)), fg);
        }
        '▜' => {
            fill(Rect::from_min_max(pos2(l, t), pos2(r, cy)), fg);
            fill(Rect::from_min_max(pos2(cx, cy), pos2(r, b)), fg);
        }
        '▝' => fill(Rect::from_min_max(pos2(cx, t), pos2(r, cy)), fg),
        '▞' => {
            fill(Rect::from_min_max(pos2(cx, t), pos2(r, cy)), fg);
            fill(Rect::from_min_max(pos2(l, cy), pos2(cx, b)), fg);
        }
        '▟' => {
            fill(Rect::from_min_max(pos2(cx, t), pos2(r, b)), fg);
            fill(Rect::from_min_max(pos2(l, cy), pos2(cx, b)), fg);
        }
        _ => return false,
    }
    true
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
        Key::I => 0x09,
        Key::J => 0x0A,
        Key::K => 0x0B,
        Key::L => 0x0C,
        Key::M => 0x0D,
        Key::N => 0x0E,
        Key::O => 0x0F,
        Key::P => 0x10,
        Key::Q => 0x11,
        Key::R => 0x12,
        Key::S => 0x13,
        Key::T => 0x14,
        Key::U => 0x15,
        Key::V => 0x16,
        Key::W => 0x17,
        Key::X => 0x18,
        Key::Y => 0x19,
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
