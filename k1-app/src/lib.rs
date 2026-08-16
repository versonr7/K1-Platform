#![no_std]
#![allow(warnings)]
#![cfg_attr(not(test), feature(lang_items))]

use core::ffi::{c_int, c_void};
use core::sync::atomic::{AtomicBool, AtomicI32, AtomicPtr, AtomicU32, Ordering};
use k1_gles::font::{draw_text, BitmapFont, Glyph};
use k1_gles::{BatchRenderer, GlContext};
use k1_math::{Color, Mat4, Rect};
use k1_sys::NativeWindow;

// ===== LOGGING =====
#[macro_export]
macro_rules! logfox {
    ($tag:expr, $msg:expr) => {
        {
            k1_sys::android_log(k1_sys::LogLevel::Info, $tag, $msg);
        }
    };
    ($tag:expr, $($arg:tt)*) => {
        {
            use core::fmt::Write;
            let mut buf = heapless::String::<256>::new();
            let _ = core::write!(buf, $($arg)*);
            k1_sys::android_log(k1_sys::LogLevel::Info, $tag, buf.as_str());
        }
    };
}

use core::mem::MaybeUninit;

// ===== STATE =====
static SELECTED: AtomicI32 = AtomicI32::new(1); // 1 = العنصر الأوسط محدد
static RUNNING: AtomicBool = AtomicBool::new(false);
static WIDTH: AtomicI32 = AtomicI32::new(0);
static HEIGHT: AtomicI32 = AtomicI32::new(0);
static FRAME_COUNT: AtomicU32 = AtomicU32::new(0);
static INITIALIZED: AtomicBool = AtomicBool::new(false);
static FRAME_LOCK: AtomicBool = AtomicBool::new(false);

static mut GL_CTX_STORAGE: MaybeUninit<GlContext> = MaybeUninit::uninit();
static GL_CTX: AtomicPtr<GlContext> = AtomicPtr::new(core::ptr::null_mut());

static mut BATCH_STORAGE: MaybeUninit<BatchRenderer<400, 600>> = MaybeUninit::uninit();
static BATCH: AtomicPtr<BatchRenderer<400, 600>> = AtomicPtr::new(core::ptr::null_mut());

// --- Font atlas ---
static FONT_ATLAS_BYTES: &[u8] = include_bytes!("../../assets/font_atlas.rgba");
const FONT_ATLAS_W: i32 = 512;
const FONT_ATLAS_H: i32 = 512;

// Auto-generated
pub const FONT_GLYPHS: [Option<Glyph>; 95] = [
    // ' '
    Some(Glyph {
        uv_x: 0.0f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '!'
    Some(Glyph {
        uv_x: 0.0625f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '"'
    Some(Glyph {
        uv_x: 0.125f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '#'
    Some(Glyph {
        uv_x: 0.1875f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '$'
    Some(Glyph {
        uv_x: 0.25f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '%'
    Some(Glyph {
        uv_x: 0.3125f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '&'
    Some(Glyph {
        uv_x: 0.375f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // "'"
    Some(Glyph {
        uv_x: 0.4375f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '('
    Some(Glyph {
        uv_x: 0.5f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ')'
    Some(Glyph {
        uv_x: 0.5625f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '*'
    Some(Glyph {
        uv_x: 0.625f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '+'
    Some(Glyph {
        uv_x: 0.6875f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ','
    Some(Glyph {
        uv_x: 0.75f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '-'
    Some(Glyph {
        uv_x: 0.8125f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '.'
    Some(Glyph {
        uv_x: 0.875f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '/'
    Some(Glyph {
        uv_x: 0.9375f32,
        uv_y: 0.0f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '0'
    Some(Glyph {
        uv_x: 0.0f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '1'
    Some(Glyph {
        uv_x: 0.0625f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '2'
    Some(Glyph {
        uv_x: 0.125f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '3'
    Some(Glyph {
        uv_x: 0.1875f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '4'
    Some(Glyph {
        uv_x: 0.25f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '5'
    Some(Glyph {
        uv_x: 0.3125f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '6'
    Some(Glyph {
        uv_x: 0.375f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '7'
    Some(Glyph {
        uv_x: 0.4375f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '8'
    Some(Glyph {
        uv_x: 0.5f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '9'
    Some(Glyph {
        uv_x: 0.5625f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ':'
    Some(Glyph {
        uv_x: 0.625f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ';'
    Some(Glyph {
        uv_x: 0.6875f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '<'
    Some(Glyph {
        uv_x: 0.75f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '='
    Some(Glyph {
        uv_x: 0.8125f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '>'
    Some(Glyph {
        uv_x: 0.875f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '?'
    Some(Glyph {
        uv_x: 0.9375f32,
        uv_y: 0.0625f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '@'
    Some(Glyph {
        uv_x: 0.0f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'A'
    Some(Glyph {
        uv_x: 0.0625f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'B'
    Some(Glyph {
        uv_x: 0.125f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'C'
    Some(Glyph {
        uv_x: 0.1875f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'D'
    Some(Glyph {
        uv_x: 0.25f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'E'
    Some(Glyph {
        uv_x: 0.3125f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'F'
    Some(Glyph {
        uv_x: 0.375f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'G'
    Some(Glyph {
        uv_x: 0.4375f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'H'
    Some(Glyph {
        uv_x: 0.5f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'I'
    Some(Glyph {
        uv_x: 0.5625f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'J'
    Some(Glyph {
        uv_x: 0.625f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'K'
    Some(Glyph {
        uv_x: 0.6875f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'L'
    Some(Glyph {
        uv_x: 0.75f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'M'
    Some(Glyph {
        uv_x: 0.8125f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'N'
    Some(Glyph {
        uv_x: 0.875f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'O'
    Some(Glyph {
        uv_x: 0.9375f32,
        uv_y: 0.125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'P'
    Some(Glyph {
        uv_x: 0.0f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'Q'
    Some(Glyph {
        uv_x: 0.0625f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'R'
    Some(Glyph {
        uv_x: 0.125f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'S'
    Some(Glyph {
        uv_x: 0.1875f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'T'
    Some(Glyph {
        uv_x: 0.25f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'U'
    Some(Glyph {
        uv_x: 0.3125f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'V'
    Some(Glyph {
        uv_x: 0.375f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'W'
    Some(Glyph {
        uv_x: 0.4375f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'X'
    Some(Glyph {
        uv_x: 0.5f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'Y'
    Some(Glyph {
        uv_x: 0.5625f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'Z'
    Some(Glyph {
        uv_x: 0.625f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '['
    Some(Glyph {
        uv_x: 0.6875f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '\\'
    Some(Glyph {
        uv_x: 0.75f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ']'
    Some(Glyph {
        uv_x: 0.8125f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '^'
    Some(Glyph {
        uv_x: 0.875f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '_'
    Some(Glyph {
        uv_x: 0.9375f32,
        uv_y: 0.1875f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '`'
    Some(Glyph {
        uv_x: 0.0f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'a'
    Some(Glyph {
        uv_x: 0.0625f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'b'
    Some(Glyph {
        uv_x: 0.125f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'c'
    Some(Glyph {
        uv_x: 0.1875f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'd'
    Some(Glyph {
        uv_x: 0.25f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'e'
    Some(Glyph {
        uv_x: 0.3125f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'f'
    Some(Glyph {
        uv_x: 0.375f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'g'
    Some(Glyph {
        uv_x: 0.4375f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'h'
    Some(Glyph {
        uv_x: 0.5f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'i'
    Some(Glyph {
        uv_x: 0.5625f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'j'
    Some(Glyph {
        uv_x: 0.625f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'k'
    Some(Glyph {
        uv_x: 0.6875f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'l'
    Some(Glyph {
        uv_x: 0.75f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'm'
    Some(Glyph {
        uv_x: 0.8125f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'n'
    Some(Glyph {
        uv_x: 0.875f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'o'
    Some(Glyph {
        uv_x: 0.9375f32,
        uv_y: 0.25f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'p'
    Some(Glyph {
        uv_x: 0.0f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'q'
    Some(Glyph {
        uv_x: 0.0625f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'r'
    Some(Glyph {
        uv_x: 0.125f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 's'
    Some(Glyph {
        uv_x: 0.1875f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 't'
    Some(Glyph {
        uv_x: 0.25f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'u'
    Some(Glyph {
        uv_x: 0.3125f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'v'
    Some(Glyph {
        uv_x: 0.375f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'w'
    Some(Glyph {
        uv_x: 0.4375f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'x'
    Some(Glyph {
        uv_x: 0.5f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'y'
    Some(Glyph {
        uv_x: 0.5625f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'z'
    Some(Glyph {
        uv_x: 0.625f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '{'
    Some(Glyph {
        uv_x: 0.6875f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '|'
    Some(Glyph {
        uv_x: 0.75f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '}'
    Some(Glyph {
        uv_x: 0.8125f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '~'
    Some(Glyph {
        uv_x: 0.875f32,
        uv_y: 0.3125f32,
        uv_w: 0.0625f32,
        uv_h: 0.0625f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
];

static mut FONT_STORAGE: MaybeUninit<BitmapFont> = MaybeUninit::uninit();
static FONT: AtomicPtr<BitmapFont> = AtomicPtr::new(core::ptr::null_mut());

// ===== JNI EXPORTS =====
#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnRenderThreadExit(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    unsafe {
        // نسقط BatchRenderer أولًا (يحتاج سياق GL ساريًا)
        if !BATCH.load(Ordering::Relaxed).is_null() {
            core::ptr::drop_in_place(BATCH_STORAGE.as_mut_ptr());
            BATCH.store(core::ptr::null_mut(), Ordering::Release);
        }
        // ثم نسقط GlContext (الذي بداخله NativeWindow وسياق EGL)
        if !GL_CTX.load(Ordering::Relaxed).is_null() {
            core::ptr::drop_in_place(GL_CTX_STORAGE.as_mut_ptr());
            GL_CTX.store(core::ptr::null_mut(), Ordering::Release);
        }
        INITIALIZED.store(false, Ordering::Release);
    }
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnCreate(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onCreate");
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnSurfaceCreated(
    _env: *mut c_void,
    _class: *mut c_void,
    surface: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native surfaceCreated");

    unsafe {
        let anw = k1_sys::ANativeWindow_fromSurface(_env, surface);
        if anw.is_null() {
            logfox!("ZAVOGLES", "ERROR: ANativeWindow_fromSurface returned null");
            return;
        }

        if let Some(win) = NativeWindow::from_raw(anw) {
            // 1. اقرأ الأبعاد قبل نقل ملكية win
            let w = win.width();
            let h = win.height();

            // 2. مرر win كقيمة (وليس كمرجع)
            match GlContext::from_window(win) {
                Ok(ctx) => {
                    GL_CTX_STORAGE.write(ctx);
                    GL_CTX.store(GL_CTX_STORAGE.as_mut_ptr(), Ordering::Release);

                    // لم نعد ننشئ BatchRenderer هنا، بل على خيط الرسم

                    WIDTH.store(w, Ordering::Release);
                    HEIGHT.store(h, Ordering::Release);
                    INITIALIZED.store(true, Ordering::Release);
                    RUNNING.store(true, Ordering::Release);

                    logfox!("ZAVOGLES", "EGL context ready: {}x{}", w, h);
                }
                Err(e) => logfox!("ZAVOGLES", "ERROR: GlContext failed: {}", e),
            }
        } else {
            logfox!("ZAVOGLES", "ERROR: NativeWindow::from_raw failed");
        }
    }
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnSurfaceChanged(
    _env: *mut c_void,
    _class: *mut c_void,
    width: i32,
    height: i32,
) {
    logfox!("ZAVOGLES", "Native surfaceChanged: {}x{}", width, height);
    WIDTH.store(width, Ordering::Release);
    HEIGHT.store(height, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnSurfaceDestroyed(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native surfaceDestroyed");
    RUNNING.store(false, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnPause(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onPause");
    RUNNING.store(false, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnResume(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onResume");
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnDestroy(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    logfox!("ZAVOGLES", "Native onDestroy");
    RUNNING.store(false, Ordering::Release);
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnTouch(
    _env: *mut c_void,
    _class: *mut c_void,
    x: f32,
    y: f32,
    action: i32,
) {
    if action == 0 {
        // ACTION_DOWN
        let w = WIDTH.load(Ordering::Acquire) as f32;
        let current = SELECTED.load(Ordering::Relaxed);

        if x < w * 0.33 {
            SELECTED.store(0.max(current - 1), Ordering::Release); // انتقل لليسار
        } else if x > w * 0.66 {
            SELECTED.store(2.min(current + 1), Ordering::Release); // انتقل لليمين
        }

        logfox!(
            "ZAVOGLES",
            "Selected category: {}",
            SELECTED.load(Ordering::Relaxed)
        );
    }
}

#[no_mangle]
pub extern "C" fn Java_com_versonr7_zavogles_ZavoglesActivity_nativeOnFrame(
    _env: *mut c_void,
    _class: *mut c_void,
) {
    if FRAME_LOCK
        .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
        .is_err()
    {
        return;
    }

    if !RUNNING.load(Ordering::Acquire) {
        FRAME_LOCK.store(false, Ordering::Release);
        return;
    }

    unsafe {
        let ctx_ptr = GL_CTX.load(Ordering::Acquire);
        if ctx_ptr.is_null() {
            FRAME_LOCK.store(false, Ordering::Release);
            return;
        }

        let ctx = &mut *ctx_ptr;

        let batch_ptr = BATCH.load(Ordering::Acquire);
        if batch_ptr.is_null() {
            if let Err(e) = ctx.make_current() {
                logfox!("ZAVOGLES", "ERROR: make_current failed: {}", e);
                FRAME_LOCK.store(false, Ordering::Release);
                return;
            }
            ctx.setup_gl_state();

            match BatchRenderer::<400, 600>::new() {
                Ok(batch) => {
                    BATCH_STORAGE.write(batch);
                    BATCH.store(BATCH_STORAGE.as_mut_ptr(), Ordering::Release);
                    logfox!("ZAVOGLES", "BatchRenderer created on render thread");
                }
                Err(e) => {
                    logfox!("ZAVOGLES", "ERROR: BatchRenderer failed: {}", e);
                    FRAME_LOCK.store(false, Ordering::Release);
                    return;
                }
            }

            // --- إبطال الخط القديم (إذا كان موجودًا) ---
            let font_ptr = FONT.load(Ordering::Acquire);
            if !font_ptr.is_null() {
                core::ptr::drop_in_place(font_ptr);
                FONT.store(core::ptr::null_mut(), Ordering::Release);
                logfox!("ZAVOGLES", "Font invalidated for re-upload");
            }
        }

        let batch = &mut *BATCH.load(Ordering::Acquire);

        let w = WIDTH.load(Ordering::Acquire) as f32;
        let h = HEIGHT.load(Ordering::Acquire) as f32;

        ctx.update_viewport(w as i32, h as i32);
        ctx.clear();

        let frame = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
        let time = (frame as f32) / 60.0;

        let matrix = Mat4::ortho(0.0, w, h, 0.0, -1.0, 1.0);

        // --- BACKGROUND ---
        batch.begin_frame();
        let pulse = libm::sinf(time * 0.3) * 0.02;
        batch.draw_quad(
            Rect::from_coords(0.0, 0.0, w, h),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(0.03 + pulse, 0.04 + pulse, 0.08 + pulse * 2.0, 1.0),
        );
        batch.end_frame(&matrix, time, 0.0, 0.0);

        // --- WAVE BAND ---
        batch.begin_frame();
        let wave_y = h * 0.30;
        let wave_height = h * 0.25;
        batch.draw_quad(
            Rect::from_coords(0.0, wave_y, w, wave_height),
            Rect::from_coords(0.0, 0.0, 1.0, 1.0),
            Color::new(0.1, 0.2, 0.4, 0.4),
        );
        batch.end_frame(&matrix, time, 5.0, 0.015);

        // --- XMB BUTTONS ---
        batch.begin_frame();
        draw_xmb_buttons(batch, w, h, time);
        batch.end_frame(&matrix, time, 0.0, 0.0);

        // --- XMB TEXT (الخط) ---
        let font_ptr2 = FONT.load(Ordering::Acquire);
        if font_ptr2.is_null() {
            match BitmapFont::from_atlas_data(
                FONT_ATLAS_BYTES,
                FONT_ATLAS_W,
                FONT_ATLAS_H,
                FONT_GLYPHS,
                32.0,
            ) {
                Ok(font) => {
                    FONT_STORAGE.write(font);
                    FONT.store(FONT_STORAGE.as_mut_ptr(), Ordering::Release);
                    logfox!("ZAVOGLES", "Font loaded OK");
                }
                Err(e) => {
                    logfox!("ZAVOGLES", "ERROR: Font init failed: {}", e);
                }
            }
        }

        // ارسم النص (يجب أن يكون بعد التحميل)
        let font_ptr_final = FONT.load(Ordering::Acquire);
        if !font_ptr_final.is_null() {
            let font = &*font_ptr_final;
            batch.begin_frame();
            draw_xmb_text(batch, font, w, h);
            batch.end_frame(&matrix, time, 0.0, 0.0);
        }

              // --- 🔬 اختبار رسم حرف 'A' ---
        // نرسمه بعد كل العناصر لكي يظهر فوقها
        let font_test_ptr = FONT.load(Ordering::Acquire);
if !font_test_ptr.is_null() {
    let font_test = &*font_test_ptr;
    batch.begin_frame();
    batch.set_texture(&font_test.atlas);
    let uv_a = Rect::from_coords(0.072, 0.142, 0.043, 0.045);
    let rect_a = Rect::from_coords(w * 0.5, h * 0.5, 100.0, 100.0);
    batch.draw_quad(rect_a, uv_a, Color::WHITE);
    batch.end_frame(&matrix, time, 0.0, 0.0);
}
      
        // --- SWAP ---
        if RUNNING.load(Ordering::Acquire) {
            if let Err(e) = ctx.swap_buffers() {
                logfox!("ZAVOGLES", "ERROR: swap_buffers: {}", e);
            }
        }
    }

    FRAME_LOCK.store(false, Ordering::Release);
}

// ===== XMB UI =====
fn draw_xmb_buttons(batch: &mut BatchRenderer<400, 600>, w: f32, h: f32, _time: f32) {
    let categories = ["Settings", "Games", "Media"];
    let y = h * 0.55;
    let spacing = w * 0.30;
    let start_x = w * 0.20;

    for (i, _cat) in categories.iter().enumerate() {
        let x = start_x + (i as f32 * spacing);
        let selected = SELECTED.load(Ordering::Acquire);
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

fn draw_xmb_text(batch: &mut BatchRenderer<400, 600>, font: &BitmapFont, w: f32, h: f32) {
    let categories = ["Settings", "Games", "Media"];
    let y = h * 0.55;
    let spacing = w * 0.30;
    let start_x = w * 0.20;
    let scale = 1.0; // جرب 1.0

    for (i, cat) in categories.iter().enumerate() {
        let x = start_x + (i as f32 * spacing);
        let text_w = font.measure_text(cat, scale);
        let text_x = x - text_w / 2.0;
        let text_y = y - 20.0;

        let selected = SELECTED.load(Ordering::Acquire);
        let is_selected = i as i32 == selected;
        let color = if is_selected {
            Color::WHITE
        } else {
            Color::new(0.6, 0.6, 0.6, 1.0)
        };

        draw_text(batch, font, cat, text_x, text_y, scale, color);
    }
}

#[cfg(not(test))]
#[lang = "eh_personality"]
extern "C" fn eh_personality() {}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    if let Some(loc) = info.location() {
        k1_sys::android_log(
            k1_sys::LogLevel::Error,
            "ZAVOGLES",
            "PANIC! (see logcat for details)",
        );
    } else {
        k1_sys::android_log(k1_sys::LogLevel::Error, "ZAVOGLES", "PANIC!");
    }
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_running() {
        RUNNING.store(true, Ordering::Relaxed);
        assert!(RUNNING.load(Ordering::Relaxed));
    }

    #[test]
    fn test_frame_lock() {
        assert!(FRAME_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_ok());
        assert!(FRAME_LOCK
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .is_err());
        FRAME_LOCK.store(false, Ordering::Release);
    }
}
