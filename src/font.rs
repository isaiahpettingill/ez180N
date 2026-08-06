// Expanded at compile time from the vendored hoard-of-bitfonts YAFF files.
// Source: https://github.com/robhagemans/hoard-of-bitfonts
// hoard-of-bitfonts represents its collected typefaces under CC0 where copyrightable.

pub const GLYPH_WIDTH: usize = 8;
pub const GLYPH_HEIGHT: usize = 8;
pub type Font = [[u8; GLYPH_HEIGHT]; 256];
pub const FONT_COUNT: usize = 24;

use ez180n_font_macro::yaff_fonts;

yaff_fonts! {
    VGA_8X8 = "fonts/vga_8x8.yaff";
    BBC_MASTER = "fonts/bbc_master.yaff";
    BBC_MASTER_INTERNATIONAL = "fonts/bbc_master_international.yaff";
    BBC_MICRO = "fonts/bbc_micro.yaff";
    SYSTEM_8X8 = "fonts/system_8x8.yaff";
    MSX_ARABIC_AX500 = "fonts/msx-arabic-ax500.yaff";
    MSX_RUSSIAN = "fonts/msx-russian.yaff";
    MSX_KOREAN = "fonts/msx-korean.yaff";
    MSX_JAPANESE_F900A = "fonts/msx-japanese-f900a.yaff";
    COLECOVISION_BOLD = "fonts/colecovision-bold.yaff";
    C64 = "fonts/c64.yaff";
    C16 = "fonts/c16.yaff";
    ATARI_CLASSIC = "fonts/atari-classic.yaff";
    ATARI_INTERNATIONAL = "fonts/atari-international.yaff";
    ATASCII = "fonts/atascii.yaff";
    ATARI_NAJM_65XE_ARABIC = "fonts/atari-najm-65xe-arabic.yaff";
    APPLE_I = "fonts/apple-i.yaff";
    AMSTRAD_CPC = "fonts/amstrad_cpc.yaff";
    AMSTRAD_PCW = "fonts/amstrad_pcw.yaff";
    ATARI_ST_8X8 = "fonts/atari-st-8x8.yaff";
    FUJITSU_FM7 = "fonts/fujitsu-fm7.yaff";
    JUPITER_ACE = "fonts/jupiter_ace.yaff";
    TRS80_DVI_8X8 = "fonts/trs80-dvi-8x8.yaff";
    RISCOS_3 = "fonts/riscos-3.yaff";
}
