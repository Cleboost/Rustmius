//! Reusable animated popover menu component (Termius style).

use crate::ui::icons::{self, IconStyle, regular};
use crate::ui::theme;
use egui::{Align2, Color32, CornerRadius, Rect, Sense, Stroke, StrokeKind, pos2, vec2};

#[derive(Clone, Copy)]
pub enum MenuIcon {
    Phosphor(&'static str, IconStyle),
    Badge(&'static str),
}

pub enum MenuEntry {
    Item {
        id: usize,
        icon: MenuIcon,
        label: &'static str,
        selected: bool,
    },
    DangerItem {
        id: usize,
        icon: MenuIcon,
        label: &'static str,
    },
    Separator,
}

/// Renders an animated Termius-style popover menu anchored below `anchor`.
/// Returns `Some(id)` when an item is clicked.
pub fn render_popover_menu(
    ctx: &egui::Context,
    salt: &str,
    anchor: Rect,
    width: f32,
    open: &mut bool,
    entries: &[MenuEntry],
) -> Option<usize> {
    let anim_t = ctx.animate_bool_with_time(egui::Id::new(salt).with("anim"), *open, 0.14);
    if anim_t <= 0.01 {
        return None;
    }

    let screen = ctx.content_rect();
    let y_offset = (1.0 - anim_t) * -6.0;
    let ideal_x = anchor.center().x - width * 0.5;
    let clamped_x = ideal_x.clamp(screen.left() + 8.0, (screen.right() - width - 8.0).max(8.0));
    let menu_pos = pos2(clamped_x, anchor.bottom() + 8.0 + y_offset);

    render_menu_at(
        ctx,
        salt,
        menu_pos,
        Some(anchor),
        width,
        anim_t,
        open,
        entries,
    )
}

/// Renders an animated Termius-style context menu at `pos` (clamped to screen bounds).
/// Returns `Some(id)` when an item is clicked.
pub fn render_context_menu(
    ctx: &egui::Context,
    salt: &str,
    pos: egui::Pos2,
    width: f32,
    open: &mut bool,
    entries: &[MenuEntry],
) -> Option<usize> {
    let anim_t = ctx.animate_bool_with_time(egui::Id::new(salt).with("anim"), *open, 0.14);
    if anim_t <= 0.01 {
        return None;
    }

    let height = menu_height(entries);
    let screen = ctx.content_rect();
    let y_offset = (1.0 - anim_t) * -4.0;
    let clamped_x = pos
        .x
        .clamp(screen.left() + 8.0, (screen.right() - width - 8.0).max(8.0));
    let clamped_y = pos.y.clamp(
        screen.top() + 8.0,
        (screen.bottom() - height - 8.0).max(8.0),
    );
    let menu_pos = pos2(clamped_x, clamped_y + y_offset);

    render_menu_at(ctx, salt, menu_pos, None, width, anim_t, open, entries)
}

fn menu_height(entries: &[MenuEntry]) -> f32 {
    let mut height = 12.0;
    for e in entries {
        height += match e {
            MenuEntry::Item { .. } | MenuEntry::DangerItem { .. } => 36.0,
            MenuEntry::Separator => 11.0,
        };
    }
    height
}

#[allow(clippy::too_many_arguments)]
fn render_menu_at(
    ctx: &egui::Context,
    salt: &str,
    menu_pos: egui::Pos2,
    anchor: Option<Rect>,
    width: f32,
    anim_t: f32,
    open: &mut bool,
    entries: &[MenuEntry],
) -> Option<usize> {
    let height = menu_height(entries);

    let mut clicked_id = None;
    let area_resp = egui::Area::new(egui::Id::new(salt).with("area"))
        .order(egui::Order::Foreground)
        .fixed_pos(menu_pos)
        .show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
            let painter = ui.painter();

            let shadow_c = Color32::from_black_alpha((28.0 * anim_t) as u8);
            painter.rect_filled(
                rect.translate(vec2(0.0, 4.0)),
                CornerRadius::same(10),
                shadow_c,
            );
            let bg = Color32::from_rgba_unmultiplied(0xF3, 0xF5, 0xF6, (248.0 * anim_t) as u8);
            painter.rect_filled(rect, CornerRadius::same(10), bg);
            painter.rect_stroke(
                rect,
                CornerRadius::same(10),
                Stroke::new(1.0_f32, theme::shade((18.0 * anim_t) as u8)),
                StrokeKind::Inside,
            );

            let mut y = rect.top() + 6.0;
            for (idx, entry) in entries.iter().enumerate() {
                match entry {
                    MenuEntry::Separator => {
                        let sep_y = y + 5.5;
                        painter.hline(
                            (rect.left() + 10.0)..=(rect.right() - 10.0),
                            sep_y,
                            Stroke::new(1.0_f32, theme::shade((16.0 * anim_t) as u8)),
                        );
                        y += 11.0;
                    }
                    MenuEntry::Item {
                        id,
                        icon,
                        label,
                        selected: _,
                    }
                    | MenuEntry::DangerItem { id, icon, label } => {
                        let is_danger = matches!(entry, MenuEntry::DangerItem { .. });
                        let is_selected = matches!(entry, MenuEntry::Item { selected: true, .. });
                        let row = Rect::from_min_size(
                            pos2(rect.left() + 6.0, y),
                            vec2(width - 12.0, 34.0),
                        );
                        let resp = ui
                            .interact(row, ui.id().with((salt, idx)), Sense::click())
                            .on_hover_cursor(egui::CursorIcon::PointingHand);
                        if resp.hovered() {
                            let hover_bg = if is_danger {
                                theme::DANGER.gamma_multiply(0.10 * anim_t)
                            } else {
                                theme::shade((14.0 * anim_t) as u8)
                            };
                            painter.rect_filled(row, CornerRadius::same(7), hover_bg);
                        }

                        let icon_c = if is_danger {
                            theme::DANGER.gamma_multiply(anim_t)
                        } else {
                            theme::TEXT_MUTED.gamma_multiply(anim_t)
                        };
                        let text_c = if is_danger {
                            theme::DANGER.gamma_multiply(anim_t)
                        } else {
                            theme::TEXT_PRIMARY.gamma_multiply(anim_t)
                        };
                        let ic_center = pos2(row.left() + 16.0, row.center().y);

                        match icon {
                            MenuIcon::Phosphor(glyph, style) => {
                                icons::paint(painter, ic_center, glyph, 15.0, icon_c, *style);
                            }
                            MenuIcon::Badge(txt) => {
                                let badge = Rect::from_center_size(ic_center, vec2(17.0, 17.0));
                                painter.rect_filled(badge, CornerRadius::same(4), icon_c);
                                painter.text(
                                    badge.center(),
                                    Align2::CENTER_CENTER,
                                    *txt,
                                    theme::semibold(9.5),
                                    Color32::WHITE.gamma_multiply(anim_t),
                                );
                            }
                        }

                        painter.text(
                            pos2(row.left() + 36.0, row.center().y),
                            Align2::LEFT_CENTER,
                            *label,
                            theme::medium(13.5),
                            text_c,
                        );

                        if is_selected {
                            icons::paint(
                                painter,
                                pos2(row.right() - 16.0, row.center().y),
                                regular::CHECK,
                                14.0,
                                icon_c,
                                IconStyle::Regular,
                            );
                        }

                        if resp.clicked() {
                            clicked_id = Some(*id);
                            *open = false;
                        }
                        y += 36.0;
                    }
                }
            }
        });

    if *open {
        let escaped = ctx.input(|i| i.key_pressed(egui::Key::Escape));
        let clicked_outside = ctx
            .input(|i| i.pointer.primary_pressed() || i.pointer.secondary_pressed())
            && ctx.input(|i| i.pointer.interact_pos()).is_some_and(|pos| {
                !area_resp.response.rect.contains(pos) && !anchor.is_some_and(|a| a.contains(pos))
            });
        if escaped || clicked_outside {
            *open = false;
        }
    }

    clicked_id
}
