//! Soft "acrylic" backdrop imitating Termius' blurred wallpaper.

use crate::ui::theme;
use egui::epaint::{Mesh, Shape};
use egui::{Color32, LayerId, Painter, Pos2, Rect, pos2};

fn radial_blob(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    const SEGMENTS: u32 = 64;
    let mut mesh = Mesh::default();
    mesh.colored_vertex(center, color);
    let rings = [(0.45_f32, 0.75_f32), (1.0, 0.0)];
    for (ring_idx, (r, a)) in rings.iter().enumerate() {
        let c = color.gamma_multiply(*a);
        for i in 0..SEGMENTS {
            let t = i as f32 / SEGMENTS as f32 * std::f32::consts::TAU;
            mesh.colored_vertex(center + egui::vec2(t.cos(), t.sin()) * radius * *r, c);
        }
        let base = 1 + ring_idx as u32 * SEGMENTS;
        for i in 0..SEGMENTS {
            let j = (i + 1) % SEGMENTS;
            if ring_idx == 0 {
                mesh.add_triangle(0, base + i, base + j);
            } else {
                let prev = base - SEGMENTS;
                mesh.add_triangle(prev + i, base + i, base + j);
                mesh.add_triangle(prev + i, base + j, prev + j);
            }
        }
    }
    painter.add(Shape::mesh(mesh));
}

pub fn paint_backdrop(ctx: &egui::Context, rect: Rect) {
    let painter = ctx.layer_painter(LayerId::background());
    painter.rect_filled(rect, 0.0, theme::BG_BASE);

    let w = rect.width();
    let at = |fx: f32, fy: f32| pos2(rect.left() + fx * w, rect.top() + fy * rect.height());
    let blob = |p: Pos2, r: f32, rgb: (u8, u8, u8), a: u8| {
        radial_blob(
            &painter,
            p,
            r * w,
            Color32::from_rgba_unmultiplied(rgb.0, rgb.1, rgb.2, a),
        );
    };

    blob(at(0.45, 0.32), 0.30, (0xE8, 0xE9, 0xEC), 200);
    blob(at(0.55, 0.62), 0.32, (0xDA, 0xEC, 0xEA), 220);
    blob(at(0.58, 0.05), 0.22, (0xDC, 0xEA, 0xE5), 200);
    blob(at(0.86, 0.78), 0.28, (0xC9, 0xCF, 0xD3), 200);
    blob(at(0.08, 0.80), 0.25, (0xD2, 0xDA, 0xDC), 160);
    blob(at(0.62, 0.85), 0.18, (0xD6, 0xEE, 0xF3), 140);
}
