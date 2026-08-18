#![no_std]
#![allow(warnings)]

use core::sync::atomic::{AtomicI32, Ordering};

use za_gles::font::{draw_text, BitmapFont};
use za_gles::BatchRenderer;
use za_math::{Color, Rect};

// ===== حالة XMB =====
pub struct XmbState {
    selected: AtomicI32,
}

impl XmbState {
    pub const fn new() -> Self {
        Self {
            selected: AtomicI32::new(1),
        }
    }

    pub fn get_selected(&self) -> i32 {
        self.selected.load(Ordering::Acquire)
    }

    pub fn set_selected(&self, index: i32) {
        self.selected.store(index, Ordering::Release);
    }

    pub fn handle_touch(&self, x: f32, screen_width: f32) {
        let current = self.get_selected();
        if x < screen_width * 0.33 {
            self.set_selected(0.max(current - 1));
        } else if x > screen_width * 0.66 {
            self.set_selected(2.min(current + 1));
        }
    }
}

// ===== خلفية XMB =====
pub fn draw_background<const V: usize, const I: usize>(
    batch: &mut BatchRenderer<V, I>,
    time: f32,
    w: f32,
    h: f32,
) {
    let pulse = libm::sinf(time * 0.3) * 0.02;
    batch.draw_quad(
        Rect::from_coords(0.0, 0.0, w, h),
        Rect::from_coords(0.0, 0.0, 1.0, 1.0),
        Color::new(0.03 + pulse, 0.04 + pulse, 0.08 + pulse * 2.0, 1.0),
    );
}

// ===== موجة XMB =====
pub fn draw_wave<const V: usize, const I: usize>(
    batch: &mut BatchRenderer<V, I>,
    _time: f32,
    w: f32,
    h: f32,
) {
    let wave_y = h * 0.30;
    let wave_height = h * 0.25;
    batch.draw_quad(
        Rect::from_coords(0.0, wave_y, w, wave_height),
        Rect::from_coords(0.0, 0.0, 1.0, 1.0),
        Color::new(0.1, 0.2, 0.4, 0.4),
    );
}

// ===== أزرار الفئات =====
pub fn draw_xmb_buttons<const V: usize, const I: usize>(
    batch: &mut BatchRenderer<V, I>,
    state: &XmbState,
    w: f32,
    h: f32,
) {
    let categories = ["Settings", "Games", "Media"];
    let y = h * 0.55;
    let spacing = w * 0.30;
    let start_x = w * 0.20;

    for (i, _cat) in categories.iter().enumerate() {
        let x = start_x + (i as f32 * spacing);
        let selected = state.get_selected();
        let is_selected = i as i32 == selected;
        let alpha = if is_selected { 1.0 } else { 0.4 };
        let color = if is_selected {
            Color::new(0.3, 0.7, 1.0, alpha)
        } else {
            Color::new(0.1, 0.3, 0.6, alpha)
        };

        let bw = w * 0.18;
        let bh = h * 0.08;

        batch.draw_quad(
            Rect::from_coords(x - bw / 2.0, y - bh / 2.0, bw, bh),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            color,
        );
    }
}

// ===== نصوص الفئات =====
pub fn draw_xmb_text<const V: usize, const I: usize>(
    batch: &mut BatchRenderer<V, I>,
    state: &XmbState,
    font: &BitmapFont,
    w: f32,
    h: f32,
) {
    let categories = ["Settings", "Games", "Media"];
    let y = h * 0.55;
    let spacing = w * 0.30;
    let start_x = w * 0.20;
    let scale = (h * 0.050) / font.line_height;

    for (i, cat) in categories.iter().enumerate() {
        let x = start_x + (i as f32 * spacing);
        let text_w = font.measure_text(cat, scale);
        let text_x = x - text_w / 2.0;
        let text_y = y - (font.line_height * scale) * 0.5;

        let selected = state.get_selected();
        let is_selected = i as i32 == selected;
        let color = if is_selected {
            Color::WHITE
        } else {
            Color::new(0.6, 0.6, 0.6, 1.0)
        };

        draw_text(batch, font, cat, text_x, text_y, scale, color);
    }
}
