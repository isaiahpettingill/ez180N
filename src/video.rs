use crate::{font, sdk};

pub const PIXEL_WIDTH: usize = sdk::FB_WIDTH * sdk::CHAR_WIDTH;
pub const PIXEL_HEIGHT: usize = sdk::FB_HEIGHT * sdk::CHAR_HEIGHT;
pub const PIXELS: usize = PIXEL_WIDTH * PIXEL_HEIGHT;

const ANSI_16: [u32; 16] = [
    0xFF000000, 0xFF800000, 0xFF008000, 0xFF808000, 0xFF000080, 0xFF800080, 0xFF008080, 0xFFC0C0C0,
    0xFF808080, 0xFFFF0000, 0xFF00FF00, 0xFFFFFF00, 0xFF0000FF, 0xFFFF00FF, 0xFF00FFFF, 0xFFFFFFFF,
];

pub fn render(chars: &[u8], pixels: &mut [u32], text_color: u8, background_color: u8, font_id: u8) {
    let fg = ansi_256_color(text_color);
    let bg = ansi_256_color(background_color);
    let glyphs = font::selected(font_id);
    for cy in 0..sdk::FB_HEIGHT {
        for cx in 0..sdk::FB_WIDTH {
            let ch = chars[cy * sdk::FB_WIDTH + cx];
            draw_char(ch, cx, cy, pixels, fg, bg, glyphs);
        }
    }
}

fn draw_char(
    ch: u8,
    cx: usize,
    cy: usize,
    pixels: &mut [u32],
    fg: u32,
    bg: u32,
    glyphs: &font::Font,
) {
    for row in 0..sdk::CHAR_HEIGHT {
        let y = cy * sdk::CHAR_HEIGHT + row;
        let start = y * PIXEL_WIDTH + cx * sdk::CHAR_WIDTH;
        for col in 0..sdk::CHAR_WIDTH {
            if row >= font::GLYPH_HEIGHT || col >= font::GLYPH_WIDTH {
                pixels[start + col] = bg;
                continue;
            }

            let bits = glyphs[ch as usize][row];
            let mask = 0x80 >> col;
            pixels[start + col] = if bits & mask != 0 { fg } else { bg };
        }
    }
}

fn ansi_256_color(index: u8) -> u32 {
    match index {
        0..=15 => ANSI_16[index as usize],
        16..=231 => {
            let cube = index - 16;
            let red = ansi_cube_component(cube / 36);
            let green = ansi_cube_component((cube % 36) / 6);
            let blue = ansi_cube_component(cube % 6);
            0xFF000000 | (red << 16) | (green << 8) | blue
        }
        232..=255 => {
            let level = 8 + u32::from(index - 232) * 10;
            0xFF000000 | (level << 16) | (level << 8) | level
        }
    }
}

fn ansi_cube_component(index: u8) -> u32 {
    if index == 0 {
        0
    } else {
        u32::from(55 + index * 40)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_palette_maps_system_cube_and_grayscale_colors() {
        assert_eq!(ansi_256_color(0), 0xFF000000);
        assert_eq!(ansi_256_color(15), 0xFFFFFFFF);
        assert_eq!(ansi_256_color(16), 0xFF000000);
        assert_eq!(ansi_256_color(21), 0xFF0000FF);
        assert_eq!(ansi_256_color(46), 0xFF00FF00);
        assert_eq!(ansi_256_color(196), 0xFFFF0000);
        assert_eq!(ansi_256_color(232), 0xFF080808);
        assert_eq!(ansi_256_color(255), 0xFFEEEEEE);
    }

    #[test]
    fn render_uses_selected_text_and_background_colors() {
        let chars = vec![sdk::CP437_FULL_BLOCK; sdk::FB_SIZE];
        let mut pixels = vec![0; PIXELS];

        render(&chars, &mut pixels, 196, 21, sdk::FONT_VGA_8X8);

        assert_eq!(pixels[0], 0xFFFF0000);
        assert_eq!(pixels[sdk::CHAR_WIDTH - 1], 0xFF0000FF);
        assert_eq!(pixels[(sdk::CHAR_HEIGHT - 1) * PIXEL_WIDTH], 0xFF0000FF);
    }

    #[test]
    fn generated_font_tables_are_selectable() {
        assert_ne!(
            font::selected(sdk::FONT_VGA_8X8)[b'A' as usize],
            font::selected(sdk::FONT_BBC_MASTER)[b'A' as usize]
        );
        assert_eq!(
            font::selected(255)[b'A' as usize],
            font::selected(sdk::FONT_VGA_8X8)[b'A' as usize]
        );
    }
}
