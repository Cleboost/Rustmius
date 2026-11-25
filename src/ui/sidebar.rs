//! Termius left navigation rail.

use crate::ui::icons::{self, IconStyle, bold, fill};
use crate::ui::theme;
use egui::{Align2, CornerRadius, Rect, Sense, Stroke, Ui, pos2, vec2};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NavSection {
    Hosts,
    Keychain,
    PortForwarding,
    Snippets,
    KnownHosts,
    Logs,
}

const ITEMS: [(NavSection, &str, &str, IconStyle); 6] = [
    (NavSection::Hosts, "Hosts", fill::HARD_DRIVES, IconStyle::Fill),
    (NavSection::Keychain, "Keychain", fill::KEY, IconStyle::Fill),
    (NavSection::PortForwarding, "Port Forwarding", fill::SHARE_FAT, IconStyle::Fill),
    (NavSection::Snippets, "Snippets", bold::BRACKETS_CURLY, IconStyle::Bold),
    (NavSection::KnownHosts, "Known Hosts", fill::GLOBE, IconStyle::Fill),
    (NavSection::Logs, "Logs", fill::CLOCK, IconStyle::Fill),
];

pub fn render_sidebar(ui: &mut Ui, active: &mut NavSection) {
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, theme::light(22));

    for (i, (section, label, glyph, style)) in ITEMS.iter().enumerate() {
        let item = Rect::from_min_size(
            pos2(rect.left() + 6.0, rect.top() + 12.0 + 45.0 * i as f32),
            vec2(rect.width() - 12.0, 34.0),
        );
        let resp = ui.interact(item, ui.id().with(("nav", i)), Sense::click()).on_hover_cursor(egui::CursorIcon::PointingHand);
        let selected = *active == *section;
        if selected {
            painter.rect_filled(item, CornerRadius::same(6), theme::shade(16));
        } else if resp.hovered() {
            painter.rect_filled(item, CornerRadius::same(6), theme::shade(9));
        }
        let cy = item.center().y;
        icons::paint(&painter, pos2(rect.left() + 24.0, cy), glyph, 16.0, theme::ICON_DARK, *style);
        painter.text(
            pos2(rect.left() + 45.0, cy),
            Align2::LEFT_CENTER,
            *label,
            theme::medium(13.0),
            theme::TEXT_SIDEBAR,
        );
        if resp.clicked() {
            *active = *section;
        }
    }

    painter.vline(rect.right() - 0.5, rect.y_range(), Stroke::new(1.0_f32, theme::SEPARATOR));
}
