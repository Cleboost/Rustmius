//! Custom Termius title bar: hamburger, Vaults/SFTP/session tabs, window controls.

use crate::i18n::{Language, tr};
use crate::storage::OsKind;
use crate::ui::icons::{self, IconStyle, fill, regular};
use crate::ui::os_icons::paint_os_mini_badge;
use crate::ui::popover_menu::{MenuEntry, MenuIcon, render_popover_menu};
use crate::ui::theme;
use egui::epaint::{Mesh, Shape};
use egui::{
    Align2, Color32, CornerRadius, Painter, PointerButton, Rect, Sense, Ui, ViewportCommand, pos2,
    vec2,
};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveTab {
    Vaults,
    Sftp,
    Settings,
    Session(Uuid),
}

pub enum TitlebarAction {
    None,
    Select(ActiveTab),
    CloseSession(Uuid),
}

pub struct SessionTabView<'a> {
    pub id: Uuid,
    pub title: &'a str,
    pub os: OsKind,
}

fn paint_gradient(painter: &Painter, rect: Rect) {
    let stops = [
        (0.0, theme::TOPBAR_LEFT),
        (0.5, theme::TOPBAR_MID),
        (1.0, theme::TOPBAR_RIGHT),
    ];
    let mut mesh = Mesh::default();
    for (t, c) in stops {
        let x = rect.left() + rect.width() * t;
        mesh.colored_vertex(pos2(x, rect.top()), c);
        mesh.colored_vertex(pos2(x, rect.bottom()), c);
    }
    for i in 0..(stops.len() as u32 - 1) {
        let b = i * 2;
        mesh.add_triangle(b, b + 1, b + 2);
        mesh.add_triangle(b + 1, b + 3, b + 2);
    }
    painter.add(Shape::mesh(mesh));
}

fn icon_button(
    ui: &mut Ui,
    painter: &Painter,
    rect: Rect,
    id: &str,
    glyph: &str,
    size: f32,
    close: bool,
) -> bool {
    let resp = ui.interact(rect, ui.id().with(id), Sense::click());
    let mut color = theme::TOPBAR_ICON;
    if resp.hovered() {
        ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        let bg = if close {
            theme::WINDOW_CLOSE_HOVER
        } else {
            theme::TOPBAR_TAB_HOVER
        };
        painter.rect_filled(rect, CornerRadius::same(6), bg);
        color = Color32::WHITE;
    }
    icons::paint(
        painter,
        rect.center(),
        glyph,
        size,
        color,
        IconStyle::Regular,
    );
    resp.clicked()
}

pub fn render_titlebar(
    ui: &mut Ui,
    active: ActiveTab,
    sessions: &[SessionTabView<'_>],
    menu_open: &mut bool,
    lang: Language,
) -> TitlebarAction {
    let s = tr(lang);
    let mut action = TitlebarAction::None;
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    let ctx = ui.ctx().clone();
    paint_gradient(&painter, rect);

    // Window drag / double-click maximize (registered first => lowest priority).
    let drag = ui.interact(rect, ui.id().with("titlebar_drag"), Sense::click_and_drag());
    if drag.drag_started_by(PointerButton::Primary) {
        ctx.send_viewport_cmd(ViewportCommand::StartDrag);
    }
    if drag.double_clicked() {
        let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }

    let left = rect.left();
    let right = rect.right();
    let cy = rect.center().y;
    let tab_top = rect.top() + 9.0;
    let tab_bot = rect.top() + 39.0;

    // Hamburger (7px left, 9px top/bottom, 4px gap to Vaults)
    let ham = Rect::from_center_size(pos2(left + 22.0, cy), vec2(30.0, 30.0));
    if icon_button(ui, &painter, ham, "tb_menu", regular::LIST, 16.0, false) {
        *menu_open = !*menu_open;
    }
    if let Some(0) = render_popover_menu(
        ui.ctx(),
        "titlebar_menu_popover",
        ham,
        170.0,
        menu_open,
        &[MenuEntry::Item {
            id: 0,
            icon: MenuIcon::Phosphor(regular::GEAR, IconStyle::Regular),
            label: s.menu_settings,
            selected: active == ActiveTab::Settings,
        }],
    ) {
        action = TitlebarAction::Select(ActiveTab::Settings);
    }

    // Vaults tab
    let vaults = Rect::from_min_max(pos2(left + 41.0, tab_top), pos2(left + 183.0, tab_bot));
    let vresp = ui
        .interact(vaults, ui.id().with("tab_vaults"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let vactive = active == ActiveTab::Vaults;
    let vfill = if vactive {
        theme::TOPBAR_TAB_ACTIVE
    } else if vresp.hovered() {
        theme::TOPBAR_TAB_HOVER
    } else {
        theme::TOPBAR_TAB_INACTIVE
    };
    painter.rect_filled(vaults, CornerRadius::same(6), vfill);
    let vtext = if vactive {
        theme::TOPBAR_TEXT
    } else {
        theme::TOPBAR_MUTED
    };
    icons::paint(
        &painter,
        pos2(left + 55.5, cy),
        fill::VAULT,
        15.0,
        vtext,
        IconStyle::Fill,
    );
    painter.text(
        pos2(left + 72.0, cy),
        Align2::LEFT_CENTER,
        s.tab_vaults,
        theme::medium(13.0),
        vtext,
    );
    icons::paint(
        &painter,
        pos2(left + 134.0, cy),
        regular::CLOUD,
        14.0,
        theme::TOPBAR_MUTED,
        IconStyle::Regular,
    );
    if vresp.clicked() {
        action = TitlebarAction::Select(ActiveTab::Vaults);
    }

    // Generic tab painter (SFTP + sessions) with clipping & horizontal scroll
    let tabs_left = left + 187.0;
    // tb_bell is centered at right - 127.0 with width 32.0 (left edge = right - 143.0).
    let tabs_right_limit = (right - 151.0).max(tabs_left);
    let plus_reserve = 36.0;
    let max_tabs_right = (tabs_right_limit - plus_reserve).max(tabs_left);
    let tabs_clip_rect = Rect::from_min_max(
        pos2(tabs_left, rect.top()),
        pos2(max_tabs_right, rect.bottom()),
    );
    let scroll_id = ui.id().with("titlebar_tabs_scroll_x");
    let mut scroll_x = ctx.data_mut(|d| d.get_temp::<f32>(scroll_id).unwrap_or(0.0));
    if ui.rect_contains_pointer(tabs_clip_rect) {
        let delta = ctx.input(|i| i.smooth_scroll_delta);
        let wheel = if delta.x.abs() > delta.y.abs() {
            delta.x
        } else {
            delta.y
        };
        if wheel != 0.0 {
            scroll_x = (scroll_x - wheel).max(0.0);
        }
    }

    let tab_painter = painter.with_clip_rect(tabs_clip_rect);
    let mut x = tabs_left - scroll_x;
    let mut total_tabs_width = 0.0_f32;
    let mut tab = |ui: &mut Ui,
                   id: egui::Id,
                   glyph: &str,
                   os: Option<OsKind>,
                   title: &str,
                   is_active: bool,
                   closable: bool|
     -> (bool, bool, f32) {
        let font = theme::medium(13.0);
        let tw = tab_painter
            .layout_no_wrap(title.to_owned(), font.clone(), Color32::WHITE)
            .size()
            .x;
        let width = if closable {
            32.0 + tw + 34.0
        } else {
            32.0 + tw + 15.0
        };
        let r = Rect::from_min_max(pos2(x, tab_top), pos2(x + width, tab_bot));
        let hit_r = r.intersect(tabs_clip_rect);
        let (resp_hovered, resp_clicked) = if hit_r.width() > 0.0 {
            let resp = ui
                .interact(hit_r, id, Sense::click())
                .on_hover_cursor(egui::CursorIcon::PointingHand);
            (resp.hovered(), resp.clicked())
        } else {
            (false, false)
        };
        let bg = if is_active {
            theme::TOPBAR_TAB_ACTIVE
        } else if resp_hovered {
            theme::TOPBAR_TAB_HOVER
        } else {
            theme::TOPBAR_TAB_INACTIVE
        };
        if hit_r.width() > 0.0 {
            tab_painter.rect_filled(r, CornerRadius::same(6), bg);
            let fg = if is_active {
                theme::TOPBAR_TEXT
            } else {
                theme::TOPBAR_MUTED
            };
            match os {
                Some(kind) if kind != OsKind::Unknown => {
                    paint_os_mini_badge(&tab_painter, pos2(x + 16.0, cy), kind);
                }
                _ => {
                    icons::paint(
                        &tab_painter,
                        pos2(x + 16.0, cy),
                        glyph,
                        14.0,
                        if is_active {
                            Color32::WHITE
                        } else {
                            Color32::from_rgb(0xDD, 0xE0, 0xE7)
                        },
                        IconStyle::Fill,
                    );
                }
            }
            tab_painter.text(pos2(x + 32.0, cy), Align2::LEFT_CENTER, title, font, fg);
        }

        let mut close_clicked = false;
        if closable && hit_r.width() > 0.0 {
            let cr = Rect::from_center_size(pos2(r.right() - 16.0, cy), vec2(20.0, 20.0));
            let hit_cr = cr.intersect(tabs_clip_rect);
            let (cresp_hovered, cresp_clicked) = if hit_cr.width() > 0.0 {
                let cresp = ui
                    .interact(hit_cr, id.with("close"), Sense::click())
                    .on_hover_cursor(egui::CursorIcon::PointingHand);
                (cresp.hovered(), cresp.clicked())
            } else {
                (false, false)
            };
            if cresp_hovered {
                tab_painter.rect_filled(cr, CornerRadius::same(4), theme::TOPBAR_TAB_HOVER);
            }
            if resp_hovered || is_active || cresp_hovered {
                icons::paint(
                    &tab_painter,
                    cr.center(),
                    regular::X,
                    12.0,
                    theme::TOPBAR_TEXT,
                    IconStyle::Regular,
                );
            }
            close_clicked = cresp_clicked;
        }
        let clicked = resp_clicked && !close_clicked;
        x += width + 4.0;
        total_tabs_width += width + 4.0;
        (clicked, close_clicked, x)
    };

    let (sftp_clicked, _, mut end_x) = tab(
        ui,
        ui.id().with("tab_sftp"),
        fill::FOLDER_SIMPLE,
        None,
        s.tab_sftp,
        active == ActiveTab::Sftp,
        false,
    );
    if sftp_clicked {
        action = TitlebarAction::Select(ActiveTab::Sftp);
    }

    for s in sessions {
        let (clicked, closed, nx) = tab(
            ui,
            ui.id().with(("tab_session", s.id)),
            fill::TERMINAL_WINDOW,
            Some(s.os),
            s.title,
            active == ActiveTab::Session(s.id),
            true,
        );
        if clicked {
            action = TitlebarAction::Select(ActiveTab::Session(s.id));
        }
        if closed {
            action = TitlebarAction::CloseSession(s.id);
        }
        end_x = nx;
    }

    let max_scroll = (total_tabs_width - 4.0 - tabs_clip_rect.width()).max(0.0);
    let clamped_scroll = scroll_x.clamp(0.0, max_scroll);
    ctx.data_mut(|d| d.insert_temp(scroll_id, clamped_scroll));

    // "+" new tab (pinned at tabs_right_limit when tabs overflow)
    let plus_cx = (end_x - 4.0 + 23.0).min(tabs_right_limit - 13.0);
    let plus = Rect::from_center_size(pos2(plus_cx, cy), vec2(26.0, 26.0));
    let presp = ui
        .interact(plus, ui.id().with("tb_plus"), Sense::click())
        .on_hover_cursor(egui::CursorIcon::PointingHand);
    let pcol = if presp.hovered() {
        theme::TOPBAR_TEXT
    } else {
        Color32::from_rgb(0x67, 0x6A, 0x79)
    };
    if presp.hovered() {
        painter.rect_filled(plus, CornerRadius::same(6), theme::TOPBAR_TAB_HOVER);
    }
    icons::paint(
        &painter,
        plus.center(),
        regular::PLUS,
        15.0,
        pcol,
        IconStyle::Regular,
    );
    if presp.clicked() {
        action = TitlebarAction::Select(ActiveTab::Vaults);
    }

    // Right side: notifications + window controls
    let btn = |dx: f32| Rect::from_center_size(pos2(right - dx, cy), vec2(32.0, 32.0));
    icon_button(
        ui,
        &painter,
        btn(127.0),
        "tb_bell",
        regular::BELL,
        15.0,
        false,
    );
    if icon_button(
        ui,
        &painter,
        btn(91.0),
        "tb_min",
        regular::MINUS,
        14.0,
        false,
    ) {
        ctx.send_viewport_cmd(ViewportCommand::Minimized(true));
    }
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    let max_glyph = if maximized {
        regular::COPY
    } else {
        regular::SQUARE
    };
    if icon_button(ui, &painter, btn(55.0), "tb_max", max_glyph, 13.0, false) {
        ctx.send_viewport_cmd(ViewportCommand::Maximized(!maximized));
    }
    if icon_button(ui, &painter, btn(19.0), "tb_close", regular::X, 14.0, true) {
        ctx.send_viewport_cmd(ViewportCommand::Close);
    }

    action
}
