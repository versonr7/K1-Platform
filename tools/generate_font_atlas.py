#!/usr/bin/env python3
from PIL import Image, ImageDraw, ImageFont
import os

FONT_PATH = "tools/DejaVuSans.ttf"
if not os.path.exists(FONT_PATH):
    FONT_PATH = "/usr/share/fonts/dejavu/DejaVuSans.ttf"

ATLAS_SIZE = 512
FONT_SIZE = 32
OUTPUT_DIR = "assets"
GLYPHS_PER_ROW = 16
START_CHAR = 32
END_CHAR = 127

def main():
    os.makedirs(OUTPUT_DIR, exist_ok=True)
    if not os.path.exists(FONT_PATH):
        print(f"Font not found: {FONT_PATH}")
        return

    font = ImageFont.truetype(FONT_PATH, FONT_SIZE)
    ascent, descent = font.getmetrics()
    
    atlas = Image.new("RGBA", (ATLAS_SIZE, ATLAS_SIZE), (0, 0, 0, 0))
    draw = ImageDraw.Draw(atlas)

    cell_w = ATLAS_SIZE // GLYPHS_PER_ROW
    cell_h = ATLAS_SIZE // GLYPHS_PER_ROW

    glyph_data = []

    for i, code in enumerate(range(START_CHAR, END_CHAR)):
        ch = chr(code)
        col = i % GLYPHS_PER_ROW
        row = i // GLYPHS_PER_ROW
        cell_x = col * cell_w
        cell_y = row * cell_h

        if ch == ' ':
            glyph_data.append({
                'char': ' ',
                'uv_x': 0.0,
                'uv_y': 0.0,
                'uv_w': 0.0,
                'uv_h': 0.0,
                'width': 0.0,
                'height': 0.0,
                'advance': float(FONT_SIZE * 0.4),
                'x_offset': 0.0,
                'y_offset': 0.0,
            })
            continue

        # bbox الحقيقي للحرف
        bbox = draw.textbbox((0, 0), ch, font=font)
        tw = bbox[2] - bbox[0]
        th = bbox[3] - bbox[1]

        # مركز الحرف داخل الخلية
        draw_x = cell_x + (cell_w - tw) // 2
        draw_y = cell_y + (cell_h - th) // 2

        # ✅ تأكد ما تطلع برا الخلية أبداً
        if draw_x < cell_x:
            draw_x = cell_x
        if draw_y < cell_y:
            draw_y = cell_y
        if draw_x + tw > cell_x + cell_w:
            draw_x = cell_x + cell_w - tw
        if draw_y + th > cell_y + cell_h:
            draw_y = cell_y + cell_h - th

        # ✅ اجعلها دائماً موجبة
        draw_x = max(0, draw_x)
        draw_y = max(0, draw_y)

        draw.text((draw_x, draw_y), ch, font=font, fill=(255, 255, 255, 255))

        # ✅ UV دائماً بين 0 و 1
        uv_x = draw_x / ATLAS_SIZE
        uv_y = (ATLAS_SIZE - draw_y - th) / ATLAS_SIZE
        uv_w = tw / ATLAS_SIZE
        uv_h = th / ATLAS_SIZE

        # ✅ advance = عرض الحرف الفعلي + 1.5 بكسل فراغ
        advance = float(tw) + 1.5

        glyph_data.append({
            'char': ch,
            'uv_x': uv_x,
            'uv_y': uv_y,
            'uv_w': uv_w,
            'uv_h': uv_h,
            'width': float(tw),
            'height': float(th),
            'advance': advance,
            'x_offset': 0.0,
            'y_offset': 0.0,
        })

    # حفظ الأطلس
    with open(os.path.join(OUTPUT_DIR, "font_atlas.rgba"), "wb") as f:
        f.write(atlas.tobytes())
    atlas.save(os.path.join(OUTPUT_DIR, "font_atlas.png"))
    print(f"Saved atlas: {ATLAS_SIZE}x{ATLAS_SIZE}")

    # كتابة font_glyphs.rs
    with open(os.path.join(OUTPUT_DIR, "font_glyphs.rs"), "w") as f:
        f.write("// Auto-generated\n")
        f.write("use k1_gles::font::Glyph;\n\n")
        f.write(f"pub const FONT_GLYPHS: [Option<Glyph>; {len(glyph_data)}] = [\n")
        for g in glyph_data:
            esc = repr(g['char'])
            f.write(f"    // {esc}\n    Some(Glyph {{\n")
            f.write(f"        uv_x: {g['uv_x']:.6}f32,\n")
            f.write(f"        uv_y: {g['uv_y']:.6}f32,\n")
            f.write(f"        uv_w: {g['uv_w']:.6}f32,\n")
            f.write(f"        uv_h: {g['uv_h']:.6}f32,\n")
            f.write(f"        width: {g['width']:.1}f32,\n")
            f.write(f"        height: {g['height']:.1}f32,\n")
            f.write(f"        advance: {g['advance']:.1}f32,\n")
            f.write(f"        x_offset: {g['x_offset']:.1}f32,\n")
            f.write(f"        y_offset: {g['y_offset']:.1}f32,\n")
            f.write("    }),\n")
        f.write("];\n")
    print("Saved font_glyphs.rs")

if __name__ == "__main__":
    main()
