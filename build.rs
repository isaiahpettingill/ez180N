use std::{env, fs, path::PathBuf};

use libyaff::{Label, YaffFont};

const FONTS: &[(&str, &str)] = &[
    ("VGA_8X8", "vga_8x8.yaff"),
    ("BBC_MASTER", "bbc_master.yaff"),
    ("BBC_MASTER_INTERNATIONAL", "bbc_master_international.yaff"),
    ("BBC_MICRO", "bbc_micro.yaff"),
    ("SYSTEM_8X8", "system_8x8.yaff"),
    ("MSX_ARABIC_AX500", "msx-arabic-ax500.yaff"),
    ("MSX_RUSSIAN", "msx-russian.yaff"),
    ("MSX_KOREAN", "msx-korean.yaff"),
    ("MSX_JAPANESE_F900A", "msx-japanese-f900a.yaff"),
    ("COLECOVISION_BOLD", "colecovision-bold.yaff"),
    ("C64", "c64.yaff"),
    ("C16", "c16.yaff"),
    ("ATARI_CLASSIC", "atari-classic.yaff"),
    ("ATARI_INTERNATIONAL", "atari-international.yaff"),
    ("ATASCII", "atascii.yaff"),
    ("ATARI_NAJM_65XE_ARABIC", "atari-najm-65xe-arabic.yaff"),
    ("APPLE_I", "apple-i.yaff"),
    ("AMSTRAD_CPC", "amstrad_cpc.yaff"),
    ("AMSTRAD_PCW", "amstrad_pcw.yaff"),
    ("ATARI_ST_8X8", "atari-st-8x8.yaff"),
    ("FUJITSU_FM7", "fujitsu-fm7.yaff"),
    ("JUPITER_ACE", "jupiter_ace.yaff"),
    ("TRS80_DVI_8X8", "trs80-dvi-8x8.yaff"),
    ("RISCOS_3", "riscos-3.yaff"),
];

const WIDTH: usize = 8;
const HEIGHT: usize = 8;
const GLYPHS: usize = 256;

type Font = [[u8; HEIGHT]; GLYPHS];

fn parse_font(path: &PathBuf) -> Font {
    let source = YaffFont::from_path(path).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error:?}", path.display());
    });
    if let Some((width, height)) = source.cell_size {
        assert!(
            width <= WIDTH as u32 && height <= HEIGHT as u32,
            "{} is larger than the 8x8 console cell",
            path.display()
        );
    }

    let mut font = [[0; HEIGHT]; GLYPHS];
    for glyph in source.glyphs {
        let Some(codepoint) = glyph.labels.iter().find_map(|label| match label {
            Label::Codepoint(values) => values.first().copied(),
            _ => None,
        }) else {
            continue;
        };
        let codepoint = usize::from(codepoint);
        if codepoint >= GLYPHS || glyph.bitmap.width > WIDTH || glyph.bitmap.height > HEIGHT {
            continue;
        }

        for (row, pixels) in glyph.bitmap.pixels.iter().take(HEIGHT).enumerate() {
            for (column, pixel) in pixels.iter().take(WIDTH).enumerate() {
                if *pixel {
                    font[codepoint][row] |= 0x80 >> column;
                }
            }
        }
    }
    font
}

fn write_font(output: &mut String, name: &str, font: &Font) {
    output.push_str(&format!("pub const {name}: Font = [\n"));
    for (index, glyph) in font.iter().enumerate() {
        output.push_str("    [");
        for (column, value) in glyph.iter().enumerate() {
            if column != 0 {
                output.push_str(", ");
            }
            output.push_str(&format!("0x{value:02X}"));
        }
        output.push_str(&format!("], // 0x{index:02X}\n"));
    }
    output.push_str("];\n\n");
}

fn main() {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let font_dir = manifest_dir.join("fonts");
    let mut output = String::new();

    for (name, filename) in FONTS {
        let path = font_dir.join(filename);
        println!("cargo:rerun-if-changed={}", path.display());
        write_font(&mut output, name, &parse_font(&path));
    }

    output.push_str("pub const FONTS: [&Font; FONT_COUNT] = [\n");
    for (name, _) in FONTS {
        output.push_str(&format!("    &{name},\n"));
    }
    output.push_str("];\n\n");
    output.push_str("pub fn selected(id: u8) -> &'static Font {\n");
    output.push_str("    FONTS.get(id as usize).copied().unwrap_or(&VGA_8X8)\n");
    output.push_str("}\n");

    let checked_in = fs::read_to_string(manifest_dir.join("src/font_tables.rs"))
        .expect("failed to read checked-in generated font tables");
    assert_eq!(
        checked_in, output,
        "font_tables.rs is out of date; regenerate it from the vendored YAFF files"
    );
}
