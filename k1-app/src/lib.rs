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
        uv_x: 0.0214844f32,
        uv_y: 1.02734f32,
        uv_w: 0.0195312f32,
        uv_h: 0.0f32,
        width: 1e+01f32,
        height: 0e+00f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '!'
    Some(Glyph {
        uv_x: 0.0800781f32,
        uv_y: 0.960938f32,
        uv_w: 0.0253906f32,
        uv_h: 0.0449219f32,
        width: 1e+01f32,
        height: 2e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '"'
    Some(Glyph {
        uv_x: 0.140625f32,
        uv_y: 0.960938f32,
        uv_w: 0.0292969f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '#'
    Some(Glyph {
        uv_x: 0.191406f32,
        uv_y: 0.960938f32,
        uv_w: 0.0527344f32,
        uv_h: 0.0449219f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '$'
    Some(Glyph {
        uv_x: 0.261719f32,
        uv_y: 0.949219f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0585938f32,
        width: 2e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '%'
    Some(Glyph {
        uv_x: 0.314453f32,
        uv_y: 0.960938f32,
        uv_w: 0.0585938f32,
        uv_h: 0.0449219f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '&'
    Some(Glyph {
        uv_x: 0.380859f32,
        uv_y: 0.960938f32,
        uv_w: 0.0488281f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // "'"
    Some(Glyph {
        uv_x: 0.458984f32,
        uv_y: 0.960938f32,
        uv_w: 0.0175781f32,
        uv_h: 0.0449219f32,
        width: 9e+00f32,
        height: 2e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '('
    Some(Glyph {
        uv_x: 0.519531f32,
        uv_y: 0.953125f32,
        uv_w: 0.0234375f32,
        uv_h: 0.0566406f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ')'
    Some(Glyph {
        uv_x: 0.582031f32,
        uv_y: 0.953125f32,
        uv_w: 0.0234375f32,
        uv_h: 0.0566406f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '*'
    Some(Glyph {
        uv_x: 0.640625f32,
        uv_y: 0.960938f32,
        uv_w: 0.03125f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '+'
    Some(Glyph {
        uv_x: 0.691406f32,
        uv_y: 0.966797f32,
        uv_w: 0.0527344f32,
        uv_h: 0.0410156f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ','
    Some(Glyph {
        uv_x: 0.771484f32,
        uv_y: 1.01172f32,
        uv_w: 0.0195312f32,
        uv_h: 0.015625f32,
        width: 1e+01f32,
        height: 8e+00f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '-'
    Some(Glyph {
        uv_x: 0.832031f32,
        uv_y: 0.998047f32,
        uv_w: 0.0234375f32,
        uv_h: 0.0195312f32,
        width: 1e+01f32,
        height: 1e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '.'
    Some(Glyph {
        uv_x: 0.896484f32,
        uv_y: 1.01562f32,
        uv_w: 0.0195312f32,
        uv_h: 0.0078125f32,
        width: 1e+01f32,
        height: 4e+00f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '/'
    Some(Glyph {
        uv_x: 0.957031f32,
        uv_y: 0.957031f32,
        uv_w: 0.0214844f32,
        uv_h: 0.0507812f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '0'
    Some(Glyph {
        uv_x: 0.0117188f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '1'
    Some(Glyph {
        uv_x: 0.0742188f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '2'
    Some(Glyph {
        uv_x: 0.136719f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '3'
    Some(Glyph {
        uv_x: 0.199219f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '4'
    Some(Glyph {
        uv_x: 0.261719f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '5'
    Some(Glyph {
        uv_x: 0.324219f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '6'
    Some(Glyph {
        uv_x: 0.386719f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '7'
    Some(Glyph {
        uv_x: 0.449219f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '8'
    Some(Glyph {
        uv_x: 0.511719f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '9'
    Some(Glyph {
        uv_x: 0.574219f32,
        uv_y: 0.898438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ':'
    Some(Glyph {
        uv_x: 0.644531f32,
        uv_y: 0.916016f32,
        uv_w: 0.0214844f32,
        uv_h: 0.0332031f32,
        width: 1e+01f32,
        height: 2e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ';'
    Some(Glyph {
        uv_x: 0.707031f32,
        uv_y: 0.912109f32,
        uv_w: 0.0214844f32,
        uv_h: 0.0410156f32,
        width: 1e+01f32,
        height: 2e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '<'
    Some(Glyph {
        uv_x: 0.753906f32,
        uv_y: 0.910156f32,
        uv_w: 0.0527344f32,
        uv_h: 0.0371094f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '='
    Some(Glyph {
        uv_x: 0.816406f32,
        uv_y: 0.921875f32,
        uv_w: 0.0527344f32,
        uv_h: 0.0292969f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '>'
    Some(Glyph {
        uv_x: 0.878906f32,
        uv_y: 0.910156f32,
        uv_w: 0.0527344f32,
        uv_h: 0.0371094f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '?'
    Some(Glyph {
        uv_x: 0.951172f32,
        uv_y: 0.898438f32,
        uv_w: 0.0332031f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '@'
    Some(Glyph {
        uv_x: 0.0f32,
        uv_y: 0.830078f32,
        uv_w: 0.0625f32,
        uv_h: 0.0546875f32,
        width: 3e+01f32,
        height: 3e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'A'
    Some(Glyph {
        uv_x: 0.0722656f32,
        uv_y: 0.835938f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'B'
    Some(Glyph {
        uv_x: 0.134766f32,
        uv_y: 0.835938f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'C'
    Some(Glyph {
        uv_x: 0.197266f32,
        uv_y: 0.835938f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'D'
    Some(Glyph {
        uv_x: 0.255859f32,
        uv_y: 0.835938f32,
        uv_w: 0.0488281f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'E'
    Some(Glyph {
        uv_x: 0.324219f32,
        uv_y: 0.835938f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'F'
    Some(Glyph {
        uv_x: 0.388672f32,
        uv_y: 0.835938f32,
        uv_w: 0.0351562f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'G'
    Some(Glyph {
        uv_x: 0.443359f32,
        uv_y: 0.835938f32,
        uv_w: 0.0488281f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'H'
    Some(Glyph {
        uv_x: 0.507812f32,
        uv_y: 0.835938f32,
        uv_w: 0.046875f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'I'
    Some(Glyph {
        uv_x: 0.583984f32,
        uv_y: 0.835938f32,
        uv_w: 0.0175781f32,
        uv_h: 0.0449219f32,
        width: 9e+00f32,
        height: 2e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'J'
    Some(Glyph {
        uv_x: 0.644531f32,
        uv_y: 0.830078f32,
        uv_w: 0.0214844f32,
        uv_h: 0.0566406f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'K'
    Some(Glyph {
        uv_x: 0.697266f32,
        uv_y: 0.835938f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'L'
    Some(Glyph {
        uv_x: 0.763672f32,
        uv_y: 0.835938f32,
        uv_w: 0.0351562f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'M'
    Some(Glyph {
        uv_x: 0.816406f32,
        uv_y: 0.835938f32,
        uv_w: 0.0546875f32,
        uv_h: 0.0449219f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'N'
    Some(Glyph {
        uv_x: 0.882812f32,
        uv_y: 0.835938f32,
        uv_w: 0.046875f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'O'
    Some(Glyph {
        uv_x: 0.943359f32,
        uv_y: 0.835938f32,
        uv_w: 0.0488281f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'P'
    Some(Glyph {
        uv_x: 0.0117188f32,
        uv_y: 0.773438f32,
        uv_w: 0.0371094f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'Q'
    Some(Glyph {
        uv_x: 0.0683594f32,
        uv_y: 0.769531f32,
        uv_w: 0.0488281f32,
        uv_h: 0.0527344f32,
        width: 2e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'R'
    Some(Glyph {
        uv_x: 0.134766f32,
        uv_y: 0.773438f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'S'
    Some(Glyph {
        uv_x: 0.199219f32,
        uv_y: 0.773438f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'T'
    Some(Glyph {
        uv_x: 0.259766f32,
        uv_y: 0.773438f32,
        uv_w: 0.0410156f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'U'
    Some(Glyph {
        uv_x: 0.320312f32,
        uv_y: 0.773438f32,
        uv_w: 0.0449219f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'V'
    Some(Glyph {
        uv_x: 0.384766f32,
        uv_y: 0.773438f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'W'
    Some(Glyph {
        uv_x: 0.4375f32,
        uv_y: 0.773438f32,
        uv_w: 0.0625f32,
        uv_h: 0.0449219f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'X'
    Some(Glyph {
        uv_x: 0.509766f32,
        uv_y: 0.773438f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'Y'
    Some(Glyph {
        uv_x: 0.572266f32,
        uv_y: 0.773438f32,
        uv_w: 0.0410156f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'Z'
    Some(Glyph {
        uv_x: 0.634766f32,
        uv_y: 0.773438f32,
        uv_w: 0.0429688f32,
        uv_h: 0.0449219f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '['
    Some(Glyph {
        uv_x: 0.707031f32,
        uv_y: 0.765625f32,
        uv_w: 0.0234375f32,
        uv_h: 0.0566406f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '\\'
    Some(Glyph {
        uv_x: 0.769531f32,
        uv_y: 0.769531f32,
        uv_w: 0.0214844f32,
        uv_h: 0.0507812f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // ']'
    Some(Glyph {
        uv_x: 0.832031f32,
        uv_y: 0.765625f32,
        uv_w: 0.0234375f32,
        uv_h: 0.0566406f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '^'
    Some(Glyph {
        uv_x: 0.878906f32,
        uv_y: 0.773438f32,
        uv_w: 0.0527344f32,
        uv_h: 0.0449219f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '_'
    Some(Glyph {
        uv_x: 0.951172f32,
        uv_y: 0.832031f32,
        uv_w: 0.0351562f32,
        uv_h: 0.015625f32,
        width: 2e+01f32,
        height: 8e+00f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '`'
    Some(Glyph {
        uv_x: 0.015625f32,
        uv_y: 0.701172f32,
        uv_w: 0.03125f32,
        uv_h: 0.0507812f32,
        width: 2e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'a'
    Some(Glyph {
        uv_x: 0.0742188f32,
        uv_y: 0.724609f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'b'
    Some(Glyph {
        uv_x: 0.136719f32,
        uv_y: 0.707031f32,
        uv_w: 0.0390625f32,
        uv_h: 0.046875f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'c'
    Some(Glyph {
        uv_x: 0.201172f32,
        uv_y: 0.724609f32,
        uv_w: 0.0351562f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'd'
    Some(Glyph {
        uv_x: 0.261719f32,
        uv_y: 0.707031f32,
        uv_w: 0.0390625f32,
        uv_h: 0.046875f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'e'
    Some(Glyph {
        uv_x: 0.324219f32,
        uv_y: 0.724609f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'f'
    Some(Glyph {
        uv_x: 0.394531f32,
        uv_y: 0.707031f32,
        uv_w: 0.0234375f32,
        uv_h: 0.046875f32,
        width: 1e+01f32,
        height: 2e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'g'
    Some(Glyph {
        uv_x: 0.449219f32,
        uv_y: 0.71875f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0488281f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'h'
    Some(Glyph {
        uv_x: 0.511719f32,
        uv_y: 0.707031f32,
        uv_w: 0.0390625f32,
        uv_h: 0.046875f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'i'
    Some(Glyph {
        uv_x: 0.583984f32,
        uv_y: 0.707031f32,
        uv_w: 0.0175781f32,
        uv_h: 0.046875f32,
        width: 9e+00f32,
        height: 2e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'j'
    Some(Glyph {
        uv_x: 0.646484f32,
        uv_y: 0.701172f32,
        uv_w: 0.0195312f32,
        uv_h: 0.0605469f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'k'
    Some(Glyph {
        uv_x: 0.699219f32,
        uv_y: 0.707031f32,
        uv_w: 0.0371094f32,
        uv_h: 0.046875f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'l'
    Some(Glyph {
        uv_x: 0.771484f32,
        uv_y: 0.707031f32,
        uv_w: 0.0175781f32,
        uv_h: 0.046875f32,
        width: 9e+00f32,
        height: 2e+01f32,
        advance: 9e+00f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'm'
    Some(Glyph {
        uv_x: 0.8125f32,
        uv_y: 0.724609f32,
        uv_w: 0.0605469f32,
        uv_h: 0.0351562f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'n'
    Some(Glyph {
        uv_x: 0.886719f32,
        uv_y: 0.724609f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'o'
    Some(Glyph {
        uv_x: 0.949219f32,
        uv_y: 0.724609f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'p'
    Some(Glyph {
        uv_x: 0.0117188f32,
        uv_y: 0.65625f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0488281f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'q'
    Some(Glyph {
        uv_x: 0.0742188f32,
        uv_y: 0.65625f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0488281f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'r'
    Some(Glyph {
        uv_x: 0.142578f32,
        uv_y: 0.662109f32,
        uv_w: 0.0273438f32,
        uv_h: 0.0351562f32,
        width: 1e+01f32,
        height: 2e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 's'
    Some(Glyph {
        uv_x: 0.201172f32,
        uv_y: 0.662109f32,
        uv_w: 0.0332031f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 't'
    Some(Glyph {
        uv_x: 0.267578f32,
        uv_y: 0.648438f32,
        uv_w: 0.0253906f32,
        uv_h: 0.0449219f32,
        width: 1e+01f32,
        height: 2e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'u'
    Some(Glyph {
        uv_x: 0.324219f32,
        uv_y: 0.662109f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'v'
    Some(Glyph {
        uv_x: 0.386719f32,
        uv_y: 0.662109f32,
        uv_w: 0.0371094f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'w'
    Some(Glyph {
        uv_x: 0.443359f32,
        uv_y: 0.662109f32,
        uv_w: 0.0507812f32,
        uv_h: 0.0351562f32,
        width: 3e+01f32,
        height: 2e+01f32,
        advance: 3e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'x'
    Some(Glyph {
        uv_x: 0.511719f32,
        uv_y: 0.662109f32,
        uv_w: 0.0371094f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'y'
    Some(Glyph {
        uv_x: 0.574219f32,
        uv_y: 0.65625f32,
        uv_w: 0.0371094f32,
        uv_h: 0.0488281f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // 'z'
    Some(Glyph {
        uv_x: 0.638672f32,
        uv_y: 0.662109f32,
        uv_w: 0.0332031f32,
        uv_h: 0.0351562f32,
        width: 2e+01f32,
        height: 2e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '{'
    Some(Glyph {
        uv_x: 0.699219f32,
        uv_y: 0.638672f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0585938f32,
        width: 2e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '|'
    Some(Glyph {
        uv_x: 0.769531f32,
        uv_y: 0.636719f32,
        uv_w: 0.0214844f32,
        uv_h: 0.0625f32,
        width: 1e+01f32,
        height: 3e+01f32,
        advance: 1e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '}'
    Some(Glyph {
        uv_x: 0.824219f32,
        uv_y: 0.638672f32,
        uv_w: 0.0390625f32,
        uv_h: 0.0585938f32,
        width: 2e+01f32,
        height: 3e+01f32,
        advance: 2e+01f32,
        x_offset: 0e+00f32,
        y_offset: 0e+00f32,
    }),
    // '~'
    Some(Glyph {
        uv_x: 0.878906f32,
        uv_y: 0.673828f32,
        uv_w: 0.0527344f32,
        uv_h: 0.0273438f32,
        width: 3e+01f32,
        height: 1e+01f32,
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
    // --- تهيئة سياق OpenGL على خيط العرض ---
    if let Err(e) = ctx.make_current() {
        logfox!("ZAVOGLES", "ERROR: make_current failed: {}", e);
        FRAME_LOCK.store(false, Ordering::Release);
        return;
    }
    ctx.setup_gl_state();

    // --- إنشاء BatchRenderer جديد ---
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

    // --- إبطال الخط القديم (إذا كان موجودًا من سياق سابق) ---
    let font_ptr = FONT.load(Ordering::Acquire);
    if !font_ptr.is_null() {
        unsafe { core::ptr::drop_in_place(font_ptr); }
        FONT.store(core::ptr::null_mut(), Ordering::Release);
        logfox!("ZAVOGLES", "Font invalidated for re-upload");
    }
}

let batch = &mut *BATCH.load(Ordering::Acquire);

// ... (باقي الكود: تحميل w, h, رسم الخلفية والموجة والأزرار والنص) ...

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
        let font_ptr = FONT.load(Ordering::Acquire);
        if font_ptr.is_null() {
            logfox!("ZAVOGLES", "Font init starting..."); // ← هنا
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
                    logfox!("ZAVOGLES", "Font loaded OK"); // ← وهنا
                }
                Err(e) => {
                    logfox!("ZAVOGLES", "ERROR: Font init failed: {}", e); // ← وهنا
                }
            }
        }

        let font_ptr2 = FONT.load(Ordering::Acquire);
        logfox!("ZAVOGLES", "font_ptr2 null? {}", font_ptr2.is_null()); // ← هنا

        if !font_ptr2.is_null() {
            let font = &*font_ptr2;
            logfox!("ZAVOGLES", "Drawing text with font OK"); // ← وهنا
            batch.begin_frame();
            draw_xmb_text(batch, font, w, h);
            batch.end_frame(&matrix, time, 0.0, 0.0);
        } else {
            logfox!("ZAVOGLES", "Font is NULL, skipping text draw"); // ← وهنا
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
    // --- اختبار: رسم الخلية الأولى من الأطلس كمربع كبير في المنتصف ---
    let test_x = w * 0.5;
    let test_y = h * 0.5;
    let test_size = 100.0;

    // UV للخلية الأولى (الحرف '!' أو المسافة) في شبكة 16×16
    let uv = Rect::from_coords(0.0, 0.0, 0.0625, 0.0625);
    let rect = Rect::from_coords(test_x - test_size / 2.0, test_y - test_size / 2.0, test_size, test_size);

    batch.set_texture(&font.atlas);
    batch.draw_quad(rect, uv, Color::WHITE);

    logfox!("ZAVOGLES", "TEST: Drew atlas cell at {},{}", test_x, test_y);
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
