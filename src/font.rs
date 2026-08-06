// Generated at build time from the vendored hoard-of-bitfonts YAFF files.
// Source: https://github.com/robhagemans/hoard-of-bitfonts
// hoard-of-bitfonts represents its collected typefaces under CC0 where copyrightable.

pub const GLYPH_WIDTH: usize = 8;
pub const GLYPH_HEIGHT: usize = 8;
pub type Font = [[u8; GLYPH_HEIGHT]; 256];
pub const FONT_COUNT: usize = 24;

include!("font_tables.rs");
