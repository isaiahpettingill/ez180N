use crate::{font, sdk};

pub const PIXEL_WIDTH: usize = sdk::FB_WIDTH * sdk::CHAR_WIDTH;
pub const PIXEL_HEIGHT: usize = sdk::FB_HEIGHT * sdk::CHAR_HEIGHT;
pub const PIXELS: usize = PIXEL_WIDTH * PIXEL_HEIGHT;

const FG: u32 = 0xFFE0E0E0;
const BG: u32 = 0xFF101018;

pub fn render(chars: &[u8], pixels: &mut [u32]) {
    for cy in 0..sdk::FB_HEIGHT {
        for cx in 0..sdk::FB_WIDTH {
            let ch = chars[cy * sdk::FB_WIDTH + cx];
            draw_char(ch, cx, cy, pixels);
        }
    }
}

fn draw_char(ch: u8, cx: usize, cy: usize, pixels: &mut [u32]) {
    for row in 0..sdk::CHAR_HEIGHT {
        let y = cy * sdk::CHAR_HEIGHT + row;
        let start = y * PIXEL_WIDTH + cx * sdk::CHAR_WIDTH;
        for col in 0..sdk::CHAR_WIDTH {
            if row >= font::GLYPH_HEIGHT || col >= font::GLYPH_WIDTH {
                pixels[start + col] = BG;
                continue;
            }

            let bits = font::VGA_8X8[ch as usize][row];
            let mask = 0x80 >> col;
            pixels[start + col] = if bits & mask != 0 { FG } else { BG };
        }
    }
}
